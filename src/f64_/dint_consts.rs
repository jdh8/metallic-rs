//! Accurate-path (dint, 128-bit) constants ported verbatim from CORE-MATH
//! binary64/log/dint.h.  Value = (-1)^sgn * (m / 2^127) * 2^ex, m normalized
//! with bit 127 set (m in [2^127, 2^128)).  GENERATED — do not edit by hand.
#![allow(clippy::unreadable_literal)]
// `LOG2_INV` and `ONE` are part of the full log-family table set but unused by the
// natural logarithm alone.
#![allow(dead_code)]

use super::dint::Dint;

pub static INVERSE_2: [Dint; 240] = [
    Dint {
        sgn: false,
        ex: 1,
        m: 0x80000000000000000000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xfe03f80fe03f80ff0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xfc0fc0fc0fc0fc100000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xfa232cf252138ac00000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xf83e0f83e0f83e100000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xf6603d980f6603da0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xf4898d5f85bb39510000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xf2b9d6480f2b9d650000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xf0f0f0f0f0f0f0f10000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xef2eb71fc43452390000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xed7303b5cc0ed7310000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xebbdb2a5c1619c8c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xea0ea0ea0ea0ea0f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe865ac7b7603a1970000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe6c2b4481cd8568a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe525982af70c880f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe38e38e38e38e38f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe1fc780e1fc780e20000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xe070381c0e0703820000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xdee95c4ca037ba580000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xdd67c8a60dd67c8b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xdbeb61eed19c59580000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xda740da740da740e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd901b2036406c80e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd79435e50d79435f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd62b80d62b80d62c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd4c77b03531dec0e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd3680d3680d3680e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd20d20d20d20d20e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xd0b69fcbd2580d0c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xcf6474a8819ec8ea0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xce168a7725080ce20000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xcccccccccccccccd0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xcb8727c065c393e10000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xca4587e6b74f032a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc907da4e871146ad0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc7ce0c7ce0c7ce0d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc6980c6980c6980d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc565c87b5f9d4d1c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc4372f855d824ca60000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc30c30c30c30c30d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc1e4bbd595f6e9480000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xc0c0c0c0c0c0c0c10000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xbfa02fe80bfa02ff0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xbe82fa0be82fa0bf0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xbd69104707661aa30000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xbc52640bc52640bd0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xbb3ee721a54d880c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xba2e8ba2e8ba2e8c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb92143fa36f5e02f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb81702e05c0b81710000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb70fbb5a19be36590000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb60b60b60b60b60c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb509e68a9b9482200000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb40b40b40b40b40c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb30f63528917c80c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb21642c8590b21650000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb11fd3b80b11fd3c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xb02c0b02c0b02c0c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xaf3addc680af3ade0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xae4c415c9882b9320000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xad602b580ad602b60000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xac7691840ac769190000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xab8f69e28359cd120000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xaaaaaaaaaaaaaaab0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa9c84a47a07f56380000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa8e83f5717c0a8e90000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa80a80a80a80a80b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa72f05397829cbc20000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa655c4392d7b73a80000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa57eb50295fad40b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa4a9cf1d968337520000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa3d70a3d70a3d70b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa3065e3fae7cd0e10000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa237c32b16cfd7730000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa16b312ea8fc377d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0xa0a0a0a0a0a0a0a10000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9fd809fd809fd80a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9f1165e7254813e30000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9e4cad23dd5f3a210000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9d89d89d89d89d8a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9cc8e160c3fb19b90000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9c09c09c09c09c0a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9b4c6f9ef03a3caa0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9a90e7d95bc609aa0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x99d722dabde58f070000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x991f1a515885fb380000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x9868c809868c80990000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x97b425ed097b425f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x97012e025c04b80a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x964fda6c0964fda70000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x95a02568095a02570000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x94f2094f2094f20a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x94458094458094460000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x939a85c40939a85d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x92f113840497889d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x924924924924924a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x91a2b3c4d5e6f80a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x90fdbc090fdbc0910000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x905a38633e06c43b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8fb823ee08fb823f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8f1779d9fdc3a2190000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8e78356d1408e7840000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8dda5202376948090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8d3dcb08d3dcb08e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8ca29c046514e0240000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8c08c08c08c08c090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8b70344a139bc75b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8ad8f2fba93868230000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8a42f8705669db470000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x89ae4089ae4089af0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x891ac73ae9819b510000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x88888888888888890000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x87f78087f78087f80000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x8767ab5f34e47ef20000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x86d905447a34acc70000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x864b8a7de6d1d6090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x85bf37612cee3c9b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x85340853408534090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x84a9f9c8084a9f9d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x84210842108421090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x839930523fbe33680000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x83126e978d4fdf3c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x828cbfbeb9a020a40000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x82082082082082090000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x81848da8faf0d2780000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x81020408102040820000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x80000000000000000000000000000000,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x80000000000000000000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xff00ff00ff00ff020000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xfe03f80fe03f80ff0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xfd08e5500fd08e560000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xfc0fc0fc0fc0fc110000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xfb18856506ddaba70000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xfa232cf252138ac10000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf92fb2211855a8660000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf83e0f83e0f83e110000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf74e3fc22c700f760000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf6603d980f6603db0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf57403d5d00f57410000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf4898d5f85bb39510000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf3a0d52cba8723370000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf2b9d6480f2b9d660000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf1d48bcee0d399fb0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf0f0f0f0f0f0f0f20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xf00f00f00f00f0100000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xef2eb71fc43452390000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xee500ee500ee50100000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xed7303b5cc0ed7310000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xec979118f3fc4da30000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xebbdb2a5c1619c8d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xeae56403ab9590100000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xea0ea0ea0ea0ea100000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe939651fe2d8d35d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe865ac7b7603a1980000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe79372e225fe30da0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe6c2b4481cd8568a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe5f36cb00e5f36cc0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe525982af70c880f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe45932d7dc52100f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe38e38e38e38e38f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe2c4a6886a4c2e110000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe1fc780e1fc780e30000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe135a9c97500e1370000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xe070381c0e0703830000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdfac1f74346c57600000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdee95c4ca037ba580000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xde27eb2c41f3d9d20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdd67c8a60dd67c8b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdca8f158c7f91ab90000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdbeb61eed19c59590000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xdb2f171df770291a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xda740da740da740f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd9ba4256c0366e920000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd901b2036406c80f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd84a598ec9151f440000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd79435e50d79435f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd6df43fca482f00e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd62b80d62b80d62d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd578e97c3f5fe5520000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd4c77b03531dec0e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd4173289870ac52f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd3680d3680d3680e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd2ba083b445250ac0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd20d20d20d20d20e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd161543e28e502750000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd0b69fcbd2580d0c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xd00d00d00d00d00e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcf6474a8819ec8ea0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcebcf8bb5b4169cc0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xce168a7725080ce20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcd712752a886d2430000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xccccccccccccccce0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcc29786c7607f9a00000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcb8727c065c393e10000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xcae5d85f1bbd6c960000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xca4587e6b74f032a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc9a633fcd967300e0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc907da4e871146ae0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc86a78900c86a78a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc7ce0c7ce0c7ce0d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc73293d789b9f8390000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc6980c6980c6980d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc5fe740317f9d00d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc565c87b5f9d4d1d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc4ce07b00c4ce07c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc4372f855d824ca70000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc3a13de60495c7740000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc30c30c30c30c30d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc2780613c0309e030000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc1e4bbd595f6e9480000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc152500c152500c20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc0c0c0c0c0c0c0c20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xc0300c0300c0300d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbfa02fe80bfa03000000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbf112a8ad278e8de0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbe82fa0be82fa0c00000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbdf59c91700bdf5b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbd69104707661aa40000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbcdd535db1cc5b7c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbc52640bc52640bd0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbbc8408cd63069a20000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbb3ee721a54d880d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xbab656100bab65620000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xba2e8ba2e8ba2e8d0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb9a7862a0ff465890000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb92143fa36f5e02f0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb89bc36ce3e0453b0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb81702e05c0b81710000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb79300b79300b7940000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb70fbb5a19be365a0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb68d31340e4307d90000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb60b60b60b60b60c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb58a485518d1e7e50000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb509e68a9b9482200000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb48a39d44685fe980000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb40b40b40b40b40c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb38cf9b00b38cf9c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb30f63528917c80c0000000000000000,
    },
    Dint {
        sgn: false,
        ex: -1,
        m: 0xb2927c29da5519d00000000000000000,
    },
];

pub static LOG_INV_2: [Dint; 240] = [
    Dint {
        sgn: true,
        ex: -1,
        m: 0xb17217f7d1cf79abc9e3b39803f2f6af,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xaf74155120c9011d046d235ee63073dc,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xad7a02e1b24efd32160864fd949b4bd3,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xab83d135dc633301ffe6607ba902ef3b,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa991713433c2b9990ba4aea614d05700,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa7a2d41ad270c9d7cd362382a7688479,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa5b7eb7cb860fb897b6a62a0dec6e072,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa3d0a93f45169a4b09594fab088c0d64,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa1ecff97c91e267b1b7efae08e597e16,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0xa00ce1092e5498c469879c5a30cd1241,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x9e304061b5fda91a04603d87b6df81ac,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x9c5710b8cbb73a42aa554b2dd4619e63,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x9a81456cec642e104d49f9aaea3cb5e0,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x98aed221a03458b6732f89321647b358,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x96dfaabd86fa1647d61188fbc94e2f14,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x9513c36876083696b5cbc416a2418011,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x934b1089a6dc93c2bf5bb3b60554e151,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x918586c5f5e4bf019f92199ed1a4bab0,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8fc31afe30b2c6dee300bf167e95da66,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8e03c24d7300395acddae1ccce247837,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8c47720791e53314762ad19415fe25a5,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8a8e1fb794b091349eb628dba173c82d,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x88d7c11e3ad53cdc8a3111a707b6de2c,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x87244c308e670a6685e005d06dbfa8f7,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8573b71682a7d21bb21f9f89c1ab80b2,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x83c5f8299e2b4091b8f6fafe8fbb68b8,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x821b05f3b01d6774db0d58c3f7e2ea1e,
    },
    Dint {
        sgn: true,
        ex: -1,
        m: 0x8072d72d903d588c7dd1b09c70c40109,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xfd9ac57bd2442180af05924d258c14c4,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xfa553f7018c966f42780a545a1b54dce,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xf7150ab5a09f27f60a470250d40ebe8e,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xf3da161eed6b9ab1248d42f78d3e65d2,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xf0a450d139366ca77c66eb6408ff6432,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xed73aa4264b0adeb5391cf4b33e42996,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xea481236f7d35bb239a767a80d6d97e6,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xe72178c0323a1a0fcc4e1653e71d9973,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xe3ffce3a2aa649238eadb651b49ac539,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xe0e30349fd1cec8203e8e1802aba24d5,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xddcb08dc0717d85c940a666c87842842,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xdab7d02231484a93bec20cca6efe2ac4,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xd7a94a92466e833ccd88bba7d0cee8df,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xd49f69e456cf1b7b7f53bd2e406e66e6,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xd19a201127d3c646279d79f51dcc7301,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xce995f50af69d863432f3f4f861ad6a8,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xcb9d1a189ab56e777d7e9307c70c0667,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xc8a5431adfb44ca6048ce7c1a75e341a,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xc5b1cd44596fa51ff218fb8f9f9ef27f,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xc2c2abbb6e5fd57003337789d592e296,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xbfd7d1dec0a8df7037eda996244bccaf,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xbcf13343e7d9ec7f2afd17781bb3afea,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xba0ec3b633dd8b0b91dc60b2b059a609,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xb730773578cb90b3aa1116c3466beb6c,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xb45641f4e350a0d4e756eba00bc33976,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xb1801859d56249de98ce51fff99479cb,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xaeadeefacaf97d379dd6e688ebb13b01,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xabdfba9e468fd6f9472ea07749ce6bd1,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xa9157039c51ebe72e164c759686a2207,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xa64f04f0b961df7854f5275c2d15c21e,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xa38c6e138e20d834d698298adddd7f30,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xa0cda11eaf46390e632438273918db7d,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x9e1293b9998c1dad3b035eae273a855c,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x9b5b3bb5f088b7685078bbe3d392be24,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x98a78f0e9ae71d8764dec34784707838,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x95f783e6e49a9cfc025004f3ef063312,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x934b1089a6dc93c2df5bb3b60554e151,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x90a22b6875c6a1f88e91aeba609c8876,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x8dfccb1ad35ca6ef9947bdb6ddcaf59a,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x8b5ae65d67db9acf7ba5168126a58b99,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x88bc74113f23def3bc5a0fe396f40f1c,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x86216b3b0b17188c363ceae88f720f1d,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x8389c3026ac3139d6adda9d2270fa1f3,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0x80f572b1363487bcedbd0b5b3479d5f2,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xfcc8e3659d9bcbf18a0cdf301431b60b,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xf7ad6f26e7ff2efc9cd2238f75f969ad,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xf29877ff388090972b020fa1820c948d,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xed89ed86a44a01ab09d49f96cb88317a,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xe881bf932af3dac32524848e3443e03f,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xe37fde37807b84e35e9a750b6b68781c,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xde8439c1dec5687c9d57da945b5d0aa6,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xd98ec2bade71e53ed0a98f2ad65bee96,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xd49f69e456cf1b7a5f53bd2e406e66e7,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xcfb6203844b3209b18cb02f33f79c16b,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xcad2d6e7b80bf915cc507fb7a3d0bf69,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xc5f57f59c7f461569a8b6997a402bf30,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xc11e0b2a8d1e0de1da631e830fd308fe,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xbc4c6c2a226399f6276ebcfb2016a433,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xb780945bab55dceab4c7bc3d32750fd9,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xb2ba75f46099cf8f243c2e77904afa76,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xadfa035aa1ed8fdd549767e410316d2b,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xa93f2f250dac67d59ad2fb8d48054add,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xa489ec199dab06f459fb6cf0ecb411b7,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x9fda2d2cc9465c526b2b9565f5355180,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x9b2fe580ac80b182011a5b944aca8705,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x968b08643409ceb9d5c0da506a088482,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x91eb89524e100d28bfd3df5c52d67e77,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x8d515bf11fb94f22a0713268840cbcbb,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x88bc74113f23def79c5a0fe396f40f19,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0x842cc5acf1d0344b6fecdfa819b96092,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xff4489cedeab2ca6e17bd40d8d9291ec,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xf639cc185088fe625066e87f2c0f733d,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xed393b1c22351281ff4e2e660317d55f,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xe442c00de2591b4ce96ab34ce0bccd10,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xdb56446d6ad8df0928112e35a60e636f,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xd273b2058de1bd4b36bbf837b4d320c6,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xc99af2eaca4c457beaf51f66692844b2,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xc0cbf17a071f80e9396ffdf76a147cc2,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xb8069857560707a70a677b4c8bec22e0,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xaf4ad26cbc8e5bef9e8b8b88a14ff0c9,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xa6988ae903f562f17e858f08597b3a68,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0x9defad3e8f732186476d3b5b45f6ca02,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0x9550252238bd2468658e5a0b811c596d,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0x8cb9de8a32ab369497c9859530a4514c,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0x842cc5acf1d0344c1fecdfa819b96094,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xf7518e0035c3dd92606d89093278a931,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xe65b9e6eed965c4f609f5fe2058d5ff2,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xd5779687d887e0ee49dda17056e45ebb,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xc4a550a4fd9a19bb3e97660a23cc5402,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xb3e4a796a5dac21307cca0bcc06c2f8e,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0xa33576a16f1f4c79121016bd904dc95a,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0x9297997c68c1f4e6610db3d4dd423bc9,
    },
    Dint {
        sgn: true,
        ex: -5,
        m: 0x820aec4f3a222397b9e3aea6c444eef6,
    },
    Dint {
        sgn: true,
        ex: -6,
        m: 0xe31e9760a5578c6df9eb2f284f31c35a,
    },
    Dint {
        sgn: true,
        ex: -6,
        m: 0xc24929464655f482da5f3cc0b3251da6,
    },
    Dint {
        sgn: true,
        ex: -6,
        m: 0xa195492cc06605194a18dff7cdb4ae33,
    },
    Dint {
        sgn: true,
        ex: -6,
        m: 0x8102b2c49ac23a8691d082dce3ddcd08,
    },
    Dint {
        sgn: true,
        ex: -7,
        m: 0xc122451c45155150b16137f09a002b0e,
    },
    Dint {
        sgn: true,
        ex: -7,
        m: 0x8080abac46f389c4662d417ced0079c9,
    },
    Dint {
        sgn: false,
        ex: 127,
        m: 0x00000000000000000000000000000000,
    },
    Dint {
        sgn: false,
        ex: 127,
        m: 0x00000000000000000000000000000000,
    },
    Dint {
        sgn: false,
        ex: -9,
        m: 0xff805515885e014e435ab4da6a5bb50f,
    },
    Dint {
        sgn: false,
        ex: -8,
        m: 0xff015358833c4762bb481c8ee1416999,
    },
    Dint {
        sgn: false,
        ex: -7,
        m: 0xbee23afc0853b6a8a89782c20df350c2,
    },
    Dint {
        sgn: false,
        ex: -7,
        m: 0xfe054587e01f1e2bf6d3a69bd5eab72f,
    },
    Dint {
        sgn: false,
        ex: -6,
        m: 0x9e75221a352ba751452b7ea62f2198ea,
    },
    Dint {
        sgn: false,
        ex: -6,
        m: 0xbdc8d83ead88d5187faa638b5e00ee90,
    },
    Dint {
        sgn: false,
        ex: -6,
        m: 0xdcfe013d7c8cbfc5632dbac46f30d009,
    },
    Dint {
        sgn: false,
        ex: -6,
        m: 0xfc14d873c1980236c7e09e3de453f5fc,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0x8d86cc491ecbfe03f1776453b7e82558,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0x9cf43dcff5eafd2f2ad90155c8a7236a,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xac52dd7e4726a456a47a963a91bb3018,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xbba2c7b196e7e224e7950f7252c163cf,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xcae41876471f5bde91d00a417e330f8e,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xda16eb88cb8df5fb28a63ecfb66e94c0,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xe93b5c56d85a9083ce2992bfea38e76b,
    },
    Dint {
        sgn: false,
        ex: -5,
        m: 0xf85186008b1532f9e64b8b7759978998,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0x83acc1acc72389785a5333c45b7f442e,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0x8b29b7751bd7073b02e0b9ee992f2372,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0x929fb17850a0b7be5b4d3807660516a4,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0x9a0ebcb0de8e848e2c1bb082689ba814,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xa176e5f5323781d2dcf935996c92e8d4,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xa8d839f830c1fb404c7343517c8ac264,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xb032c549ba861d83774e27bc92ce3373,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xb78694572b5a5cd324cdcf68cdb2067c,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xbed3b36bd89664197c0644d7d9ed08b4,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xc61a2eb18cd907a1e5a1532f6d5a1ac1,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xcd5a1231019d66d7761e3e7b171e44b2,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xd49369d256ab1b1f9e9154e1d5263cda,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xdbc6415d876d08393e33c0c9f8824f54,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xe2f2a47ade3a18a8a0bf7c0b0d8bb4ef,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xea189eb3659aeaeb93b2a3b21f448259,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xf1383b7157972f48543fff0ff4f0aaf1,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xf85186008b1533025e4b8b7759978993,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xff64898edf55d548428ccfc99271dffa,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x8338a89652cb714ab247eb86498c2ce7,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x86bbf3e68472cb2f0b8bd20615747126,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x8a3c2c233a1563419027c74fe0e6f64f,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x8db956a97b3d0143f023472cd739f9e1,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x913378c852d65be6977e3013d10f7525,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x94aa97c0ffa91a5d4ee3880fb7d34429,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x981eb8c723fe97f21f1c134fb702d433,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x9b8fe100f47ba1d804b62af189fcba0d,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x9efe158766314e4f4d71827efe892fc8,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xa2695b665be8f3384eca87c3f0f06211,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xa5d1b79cd2af2aca8837986ceabfbed6,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xa9372f1d0da1bd10580eb71e58cd36e5,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xac99c6ccc1042e943dd557528315838d,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xaff983853c9e9e405f105039091dd7f5,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xb3566a13956a86f4471b1e1574d9fd55,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xb6b07f38ce90e4637bb2e265d0de37e1,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xba07c7aa01bd264843f9d57b324bd05f,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xbd5c481086c848dbbb596b5030403242,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xc0ae050a1abf56ad2f7f8c5fa9c50d76,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xc3fd03290648847d30480bee4cbbd698,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xc74946f4436a054ef4f5cb531201c0d3,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xca92d4e7a2b5a3adc983a9c5c4b3b135,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xcdd9b173efdc1aaa8863e007c184a1e7,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xd11de0ff15ab18c6d88d83d4cc613f21,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xd45f67e44178c6125486e73c615158b4,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xd79e4a7405ff96c31300c9be67ae5da0,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xdada8cf47dad236ddffb833c3409ee7e,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xde1433a16c66b14cde744870f54f0f18,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xe14b42ac60c605124e38eb8092a01f06,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xe47fbe3cd4d10d5b2ec0f797fdcd125c,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xe7b1aa704e2ee240b40faab6d2ad0841,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xeae10b5a7ddc8ad8806b2fc9a8038790,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xee0de5055f63eb0190a33316df83ba5a,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xf1383b7157972f4ab43fff0ff4f0aaf1,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xf460129552d2ff41e62e3201bb2bbdce,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xf7856e5ee2c9b28a76f2a1b84190a7dc,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xfaa852b25bd9b833a6dbfa03186e0666,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xfdc8c36af1f154680a3361bca696504a,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x8073622d6a80e631e897009015316073,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x82012ca5a68206d58fde85afdd2bc88a,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x838dc2fe6ac868e71a3fcbdef40100cb,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x851927139c871af867bd00c38061c51f,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x86a35abcd5ba59015481c3cbd925ccd2,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x882c5fcd7256a8c139055a6598e7c29e,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x89b438149d4582f534531dba493eb5a6,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x8b3ae55d5d30701ac63eab8837170480,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x8cc0696ea11b7b3694361c9a28d38a6a,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x8e44c60b4ccfd7dc1473aa01c7778679,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x8fc7fcf24517946a380cbe769f2c6793,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x914a0fde7bcb2d0ec429ed3aea197a60,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x92cb0086fbb1cf75a29d47c50b1182d0,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x944ad09ef4351af1a49827e081cb16ba,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x95c981d5c4e924ea45404f5aa577d6b4,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x974715d708e984dd6648d42840d9e6fb,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x98c38e4aa20c27d2846767ec990d7333,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x9a3eecd4c3eaa6aedb3a7f6e6087b947,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x9bb93315fec2d7907f589fba0865790f,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x9d3262ab4a2f4e37a1ae6ba06846fae0,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0x9eaa7d2e0fb87c35ff472bc6ce648a7d,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa0218434353f1de4d493efa632530acc,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa197795027409daa1dd1d4a6df960357,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa30c5e10e2f613e49bd9bd99e39a20b3,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa4803402004e865c31cbe0e8824116cd,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa5f2fcabbbc506d868ca4fb7ec323d74,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa764b99300134d790d04d10474301862,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xa8d56c396fc1684c01eb067d578c4756,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xaa45161d6e93167b9b081cf72249f5b2,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xabb3b8ba2ad362a11db6506cc17a01f5,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xad215587a67f0cdfe890422cb86b7cb1,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xae8dedfac04e5282ac707b8ffc22b3e8,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xaff983853c9e9e3fc5105039091dd7f8,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb1641795ce3ca978faf915300e517393,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb2cdab981f0f940bc857c77dc1df600f,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb43640f4d8a5761ff5f080a71c34b25d,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb59dd911aca1ec481d2664cf09a0c1bf,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb70475515d0f1c5e4c98c6b8be17818d,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xb86a1713c491aeaad37ee2872a6f1cd6,
    },
];

pub static P_2: [Dint; 13] = [
    Dint {
        sgn: false,
        ex: -4,
        m: 0x99df88a0430813caa1cffb6e966a70f6,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xaaa02d43f696c3e44dbe754667b6bc48,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xba2e7a1eaf85617470e5c5a5ebbe0226,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xccccccb9ec017492f934e28d924e76d4,
    },
    Dint {
        sgn: false,
        ex: -4,
        m: 0xe38e38e3807cfa4bc976e6cbd22e203f,
    },
    Dint {
        sgn: true,
        ex: -4,
        m: 0xfffffffffff924cc05b308e39fa7dfb5,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0x924924924924911d862bc3d33abb3649,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xaaaaaaaaaaaaaaaa6637fd4b19743eec,
    },
    Dint {
        sgn: false,
        ex: -3,
        m: 0xccccccccccccccccccc2ca18b08fe343,
    },
    Dint {
        sgn: true,
        ex: -3,
        m: 0xffffffffffffffffffffff2245823ae0,
    },
    Dint {
        sgn: false,
        ex: -2,
        m: 0xaaaaaaaaaaaaaaaaaaaaaaaaa5c48b54,
    },
    Dint {
        sgn: true,
        ex: -2,
        m: 0xffffffffffffffffffffffffffffebd8,
    },
    Dint {
        sgn: false,
        ex: 0,
        m: 0x80000000000000000000000000000000,
    },
];

pub static LOG2: Dint = Dint {
    sgn: false,
    ex: -1,
    m: 0xb17217f7d1cf79abc9e3b39803f2f6af,
};
pub static LOG2_INV: Dint = Dint {
    sgn: false,
    ex: 12,
    m: 0xb8aa3b295c17f0bbbe87fed0691d3e89,
};
pub static ONE: Dint = Dint {
    sgn: false,
    ex: 0,
    m: 0x80000000000000000000000000000000,
};
pub static M_ONE: Dint = Dint {
    sgn: true,
    ex: 0,
    m: 0x80000000000000000000000000000000,
};
pub static ZERO: Dint = Dint {
    sgn: false,
    ex: 0,
    m: 0x00000000000000000000000000000000,
};
