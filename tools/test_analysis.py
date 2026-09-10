#!/usr/bin/env python3
"""Small synthetic checks; run with `python3 tools/test_analysis.py`."""

import contextlib
import io
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import analysis as a


def main():
    match = a.symbol_matcher("sym:asinpi_accurate")
    assert match("_RNvNtC123_8metallic15asinpi_accurate")
    assert not match("_RNCNvNtC123_8metallic15asinpi_accurate0B7_")
    program = a.Program("""
entry:
    cmpl $0, %eax
    jne .Lchosen
    ud2
.Lchosen:
    callq helper
    callq external@PLT
    movq .Ltable(%rip), %rax
    retq
helper:
    addl $1, %eax
    retq
.section .rodata
.Ltable:
    .quad 1, 2
.Lduplicate:
    .quad 1
    .quad 2
""")
    side = a.Side([program])
    symbols = [(100, 7, ["entry"]), (200, 2, ["helper"])]
    listings = {
        100: list(zip(range(100, 107), ["cmp", "jne", "ud2", "call", "call", "mov", "ret"])),
        200: [(200, "add"), (201, "ret")],
    }
    with patch.object(a, "text_symbols", return_value=symbols), \
            patch.object(a, "disassemble", return_value=listings):
        binary = a.Binary("unused")
        mapped, following = binary.mapping(side, 100, ["entry"])
        assert mapped[100] == (program, 0) and following[101] == 102
        path = a.traced_path(side, binary, [100, 101, 103, 200, 201, 104, 300, 301, 105, 106])
        assert [i.split()[0] for i in path["insns"]] == [
            "cmpl", "jne", "addl", "callq", "movq", "retq"]
        assert path["calls"] == ["external"]
        assert path["branches"] == [(program, ".Lchosen", 2, True)]
        assert a.leg_start(side, path, "branch:0") == (program, 2)
        assert a.leg_start(side, path, "branch:jne:-1") == (program, 2)
        assert a.table_bytes(side, [path]) == 16
        path["symbols"].add((program, ".Lduplicate"))
        assert a.table_bytes(side, [path]) == 16

        leg = a.traced_path(side, binary, [103, 200, 201, 104, 300, 105, 106],
                            leg=lambda target: target == "helper")
        assert leg["calls"] == ["helper", "external"] and "leg-on-path" in leg["flags"]
        assert "addl\t$1, %eax" not in leg["insns"]

    # The trace driver must pass the function name before its three operands.
    with patch.object(a, "run", return_value=SimpleNamespace(
            stdout='TRACE {"metallic_exp": [100, 101]}\n', stderr="")) as run:
        assert a.trace("probe", "exp", {"metallic_exp": 100}, (1.7, 0.7, 0.3)) == {
            "metallic_exp": [100, 101]}
        command = run.call_args.args[0]
        assert command[command.index("--args") + 1:] == [
            "probe", "call", "exp", "1.7", "0.7", "0.3"]

    loop = a.Program("""
entry:
.Lloop:
    addl $1, %eax
    cmpl $10, %eax
    jne .Lexit
    jmp .Lloop
.Lexit:
    retq
""")
    path = a.walk(a.Side([loop]), loop, 0)
    assert "loop" in path["flags"] and path["insns"].count("addl\t$1, %eax") == 1
    assert path["insns"][-1] == "retq"
    by_mnemonic = a.walk(a.Side([loop]), loop, 0, take=["jne"])
    assert "loop" not in by_mnemonic["flags"] and "take" not in by_mnemonic["flags"]
    calls = a.Program("""
entry:
    callq first
    callq second
    retq
first:
    jne .Lwrong
    retq
.Lwrong:
    ud2
second:
.Lrepeat:
    addl $1, %eax
    jmp .Lrepeat
""")
    path = a.walk(a.Side([calls]), calls, 0)
    assert "loop" in path["flags"] and "trap" not in path["flags"]
    dispatch = a.Program("""
entry:
    leaq .LJTI0_0(%rip), %rcx
    vmulsd %xmm0, %xmm0, %xmm0
    movslq (%rcx,%rax,4), %rax
    vmulsd %xmm1, %xmm1, %xmm1
    addq %rcx, %rax
    vmulsd %xmm2, %xmm2, %xmm2
    vmulsd %xmm3, %xmm3, %xmm3
    jmpq *%rax
.LBB0_1:
    addl $1, %eax
    retq
.LBB0_2:
    ud2
.section .rodata
.LJTI0_0:
    .long .LBB0_1-.LJTI0_0
    .long .LBB0_2-.LJTI0_0
""")
    side = a.Side([dispatch])
    path = a.walk(side, dispatch, 0)
    assert "dispatch" in path["flags"] and "indirect" not in path["flags"]
    assert path["insns"][-2:] == ["addl\t$1, %eax", "retq"]
    assert a.table_bytes(side, [path]) == 8
    unknown = a.Program("entry:\n jmpq *%rax\n")
    assert "indirect" in a.walk(a.Side([unknown]), unknown, 0)["flags"]
    tail = a.Program("entry:\n jmp external@PLT\n")
    side = a.Side([tail])
    with patch.object(a, "text_symbols", return_value=[(100, 1, ["entry"])]), \
            patch.object(a, "disassemble", return_value={100: [(100, "jmp")]}):
        traced = a.traced_path(side, a.Binary("unused"), [100, 200])
    for path in (a.walk(side, tail, 0), traced):
        assert path["insns"] == ["callq\texternal@PLT"]
        assert path["calls"] == ["external"] and "opaque-tail" in path["flags"]
    assert a.directive_bytes(".ascii", r'"a\000b"') == 3
    assert a.strip_comment('.ascii "a#b" # comment') == '.ascii "a#b" '
    encoded = a.Program("entry:\n rep bsfq %r10, %rdx\n retq\n")
    assert a.align(encoded, 0, [(0, "tzcntq"), (5, "retq")]) == {0: 0, 5: 1}
    assert a.align(a.Program("entry:\n rep movsb\n"), 0, [(0, "tzcntq")]) is None
    relaxed = a.Program("entry:\n movq signgam@GOTPCREL(%rip), %rdx\n retq\n")
    assert a.align(relaxed, 0, [(0, "leaq"), (7, "retq")]) == {0: 0, 7: 1}
    assert a.align(a.Program("entry:\n movq (%rip), %rdx\n"), 0, [(0, "leaq")]) is None

    with tempfile.TemporaryDirectory() as directory:
        source = Path(directory) / "sample.rs"
        source.write_text("fn accurate() {\n    refine();\n}\n")
        coverage = a.parse_lcov(f"SF:{source}\nDA:1,7\nDA:2,7\nDA:2,2\n"
                               "BRDA:2,0,0,3\nBRDA:2,0,1,-\nend_of_record\n")
        assert a.anchor_count(coverage, "sample.rs: fn accurate() {") == 7
        assert a.anchor_count(coverage, "sample.rs: refine();") == 9
        with patch.dict(a.FN, {"exp": {
                "m": {"cov": ["sample.rs: fn accurate() {"]},
                "c": {"cov": [{"source": "sample.rs: fn accurate() {",
                                "offset": 1, "branch": [0, 0]}]}}}):
            assert a.leg_counts(coverage, "exp") == (7, 3)
        assert a.anchor_count(coverage, {"source": "sample.rs: fn accurate() {",
                                         "offset": 1}) == 9
        assert a.anchor_count(coverage, {"source": "sample.rs: refine();",
                                         "branch": [0, 1]}) == 0
        source.write_text("refine();\nrefine();\n")
        with contextlib.redirect_stderr(io.StringIO()):
            try:
                a.anchor_count(coverage, "sample.rs: refine();")
            except SystemExit as error:
                assert error.code == 2
            else:
                raise AssertionError("ambiguous coverage anchors must fail")
    print("analysis synthetic checks passed")


if __name__ == "__main__":
    main()
