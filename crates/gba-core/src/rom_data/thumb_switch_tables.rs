//! EUR Thumb switch-table roots and destinations, authenticated against original readers.
use super::{script39_registry_poses::PriorSources, Resource, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const TABLE_COUNT: usize = 23;
pub const TOTAL_DESTINATIONS: usize = 696;
pub const SOURCE_ROLE: &str = "thumb-switch-table";

#[derive(Clone, Copy, Debug)]
pub struct TableSpec {
    pub root: u32,
    pub count: usize,
    pub guard: u32,
    pub reader: u32,
    pub mov: u32,
    pub special: bool,
    pub code_sha256: &'static str,
    pub source_sha256: &'static str,
}
impl TableSpec {
    pub fn end(self) -> u32 { self.root + 4 + (self.count as u32) * 4 }
}

// Physical source bounds follow the original unsigned guards and switch readers.
pub const TABLES: [TableSpec; TABLE_COUNT] = [
    TableSpec { root: 0x11974, count: 12, guard: 0x11962, reader: 0x11968, mov: 0x11970, special: false, code_sha256: "59922697cd7c5b999b2c025c6d8db0fa79186d12fae3f73491f5c55c2d2fd176", source_sha256: "0fcbc255633bf674f40d3b4490b308616ff380ad0972041bc422b4efcf7137cd" },
    TableSpec { root: 0x12274, count: 89, guard: 0x1225e, reader: 0x12266, mov: 0x1226e, special: false, code_sha256: "af183d688b81e67335b2b181211eef1c01fee908874fa75d05ce184e58ac7ccd", source_sha256: "75f55193727469a7b08ca467b1575d38ac5c12cfaf8f01ab442c90b7aaafc631" },
    TableSpec { root: 0x14198, count: 89, guard: 0x14186, reader: 0x1418c, mov: 0x14194, special: false, code_sha256: "83d06bcfa111d28ca57661f5fd57abe0d10f91d00f68cb420b303ac04ad7a4ee", source_sha256: "53d6270dd6e6da068c2d4bbe3fe51e058b98ed9e7db20364d79c377aac055f29" },
    TableSpec { root: 0x14620, count: 89, guard: 0x14610, reader: 0x14616, mov: 0x1461e, special: false, code_sha256: "310cb4be627e091c1883071c06ff42df4d36f1854a23907abf461f7bbeb71283", source_sha256: "16ce6b8e1cf693e0b47c098c0c842df68b052fbfb8f26fe45d49c8bec943fbb6" },
    TableSpec { root: 0x191a4, count: 26, guard: 0x1918e, reader: 0x19194, mov: 0x1919c, special: false, code_sha256: "ad5ccfe50cf5f12bb12977b8b34995d1a9ff175c8baff7f48f3cd24f3b866789", source_sha256: "f9c9da087b11767f4da823de96698750b39143a50f3ae9564eba28a0548fd61d" },
    TableSpec { root: 0x22270, count: 8, guard: 0x22260, reader: 0x22266, mov: 0x2226e, special: false, code_sha256: "6d99c59be5d57735e837629376b8c1f765fede91065808c0b1d1197d0a19b8cf", source_sha256: "ee06cd4896eebf1176858b7e755b769cf3cf88ac0f29939b1e08e6530a22163e" },
    TableSpec { root: 0x25408, count: 35, guard: 0x253f2, reader: 0x253f8, mov: 0x25400, special: false, code_sha256: "f6fb37231d76f5a137d43e0dfc982b11e218f832123f79214f32de787c447941", source_sha256: "87f3de9fb6e26146f1418bc108ca391cefddb56bbe5cc52e6d9c68bb43ddc375" },
    TableSpec { root: 0x9be04, count: 19, guard: 0x9bdf4, reader: 0x9bdf8, mov: 0x9be00, special: false, code_sha256: "6634b59d5f05d4423b5d5a8223ee32aa7902534e246c672ab74353d3a828679e", source_sha256: "a66318b5fed9d1a45150c9d167c0f1b00a88a987e0fc131bffd2eb7033916a08" },
    TableSpec { root: 0x9be88, count: 12, guard: 0x9be76, reader: 0x9be7c, mov: 0x9be84, special: false, code_sha256: "886ed2fca237a27fbbe0062d52390a2f9928af0bd23cee76e4a06b5c57a30470", source_sha256: "7a88424b2ccfa9821d1e69c8ac08633bae4079e698776701ba22e5facab527ed" },
    TableSpec { root: 0x9bf10, count: 5, guard: 0x9bf02, reader: 0x9bf06, mov: 0x9bf0e, special: false, code_sha256: "c02b96ce64e8d5e38e0bbb82211fca44050b1674605f05f7632e3192a7454dd4", source_sha256: "bb9ef2a5d28b5014892019e382713c502ab57a10d22f37364cce03c183384d8a" },
    TableSpec { root: 0xa114c, count: 24, guard: 0xa113a, reader: 0xa1140, mov: 0xa1148, special: false, code_sha256: "e79a95ad3eaae421fe55833d517c8d6ed51350b69f54f71da9efae746d1b1dfb", source_sha256: "2c11c5826f7988bd308878916bdc3715efa2235021c70118432653a256db7b08" },
    TableSpec { root: 0xa556c, count: 8, guard: 0xa555c, reader: 0xa5562, mov: 0xa556a, special: false, code_sha256: "c5cd35f6517a1c75fb44f1d9bca5df6406fc74d9ec09cc8065b3f49e05b05586", source_sha256: "78b2b5f4c31449bec11d12a942adaa89c54a8a83d8c3987fa6ccb0e7b13b7e68" },
    TableSpec { root: 0xa8fd0, count: 19, guard: 0xa8fc2, reader: 0xa8fc6, mov: 0xa8fce, special: false, code_sha256: "6ea046ca917681ac2a8fc3682dfbfeb060c7b467632a6beb44581a5c899a7492", source_sha256: "03d569b83acca2e50491a3a096d3353cf64c5b455f0f5a64d738b80b52d4fcac" },
    TableSpec { root: 0xa9548, count: 37, guard: 0xa9524, reader: 0xa952c, mov: 0xa9534, special: false, code_sha256: "b5c40ab887b3de00551d36ccc53d66971ddadba29538f7dd361909e28e3318b8", source_sha256: "ebeedcbfcc0aa6116e6b453f4a700c11aae72e0233eb2f385436e16f69ade025" },
    TableSpec { root: 0xac01c, count: 8, guard: 0xac00c, reader: 0xac010, mov: 0xac018, special: false, code_sha256: "360c48a8f1d1490b295373d5e2a33339a38b052ab6a48538fbbb31a818599119", source_sha256: "f1c1b6378870b926f54d33f5960750fb4a9f480be9698ef5cfe0d0970f35ce1f" },
    TableSpec { root: 0xb94dc, count: 32, guard: 0xb94c6, reader: 0xb94ce, mov: 0xb94d6, special: false, code_sha256: "757e8321aa082826e8d864744fdc1fdac539179ba795dcb47d1c5575c35555c1", source_sha256: "a1b82940022660bc62bbe344fddd811bac3814f45f3564d9539e9726b66592cd" },
    TableSpec { root: 0xbf058, count: 13, guard: 0xbf046, reader: 0xbf04c, mov: 0xbf054, special: false, code_sha256: "926a1ebf9de30219f028c1ed976183c3e8aa8737e11597728ce4a16bfa3148d2", source_sha256: "e83f297f411db837aebad1cbb1e9aa46370395cdd0e7fbd22f5b85cec4847a87" },
    TableSpec { root: 0xcb814, count: 69, guard: 0xcb804, reader: 0xcb80a, mov: 0xcb812, special: false, code_sha256: "572f7a74ba62f8013bafd3c0a9c889ba6b3ece86940790054eaeb89c066badd0", source_sha256: "3a7ae3142d6c62d901c9a503b28e271f583e1f1ddd34c1b8fdf82cbf327d3e80" },
    TableSpec { root: 0xcbef4, count: 48, guard: 0xcbee2, reader: 0xcbee0, mov: 0xcbeec, special: true, code_sha256: "97f2ab10a3f5bd0f0e0d2232d3232d6c16abd0e33a16eea15c95f0e649992873", source_sha256: "d9df46574e8482ebfa3f7cea06952b67b64dcc376bad6ae8795cdbe64932b169" },
    TableSpec { root: 0xdb2cc, count: 6, guard: 0xdb2bc, reader: 0xdb2c0, mov: 0xdb2c8, special: false, code_sha256: "22b279bba637729dfe469127b84971a92ddd7a5abf1762a0f4c364d2188fee1d", source_sha256: "5450ddf61bd8b11b67f11b7d486936c68aae387e73c7795b1155d872398f4e31" },
    TableSpec { root: 0xe3188, count: 23, guard: 0xe317a, reader: 0xe317e, mov: 0xe3186, special: false, code_sha256: "d3667cfc57101f574d7ff28c570a482247dc6afb88cf11022050c6e5d57bea48", source_sha256: "39ca2fd32c9f6b5f7568fa9745363b65aab8aabedce8caec76774c65b7424fc4" },
    TableSpec { root: 0x12542c, count: 5, guard: 0x12541e, reader: 0x125422, mov: 0x12542a, special: false, code_sha256: "0b969313d12d651450cb7b549e091ff01778aec691d419b137c9ea411a4688ff", source_sha256: "22de0b44c9765d5c1255bcaf6077122ca1781ed5e907a3aa5a28189616bfbda2" },
    TableSpec { root: 0x1380a0, count: 20, guard: 0x13808e, reader: 0x138094, mov: 0x13809c, special: false, code_sha256: "b6024e25c9e2dbd748be5c1e6d2aa361eafd5e0ce4c85de063a10c53216fe773", source_sha256: "a7f5c582c87f00f15eb7a05553eb0040dc5caaf385e7ab16ee4eaa8dd2653bb7" },
];


/// Additional tables use roots owned by Thumb PC literals.
/// Only destinations are new sources; guest out-of-range behavior remains live.
pub const PREVIOUS_ADDITIONAL_TABLE_COUNT: usize = 32;
pub const FIRAGA_ADDITIONAL_TABLE_COUNT: usize = 34;
pub const WHITE_FLOWERS_ADDITIONAL_TABLE_COUNT: usize = 35;
pub const WHITE_FLOWERS_RESIDUAL_TABLE_COUNT: usize = 42;
pub const WHITE_FLOWERS_MIXED_TABLE_COUNT: usize = 47;
const PRE_TITLE_ADDITIONAL_TABLE_COUNT: usize = 50;
const TITLE_ADDITIONAL_TABLE_COUNT: usize = 52;
pub const WORLD_NODE_ADDITIONAL_TABLE_COUNT: usize = 53;
pub const ADDITIONAL_TABLE_COUNT: usize = 55;
const CACHED_OBJECT_ADDITIONAL_TABLE_COUNT: usize =
    ADDITIONAL_TABLE_COUNT + CACHED_OBJECT_TABLES.len();
const INDEXED_EXCHANGE_ADDITIONAL_TABLE_COUNT: usize =
    CACHED_OBJECT_ADDITIONAL_TABLE_COUNT + INDEXED_EXCHANGE_TABLES.len();
const FIVE_BYTE_LIST_ADDITIONAL_TABLE_COUNT: usize =
    INDEXED_EXCHANGE_ADDITIONAL_TABLE_COUNT + FIVE_BYTE_LIST_TABLES.len();
const BACKGROUND_VARIANT_STATE_TABLE_COUNT: usize = FIVE_BYTE_LIST_ADDITIONAL_TABLE_COUNT + 1;
pub const ADDITIONAL_DESTINATION_COUNT: usize = 1750;
pub const ADDITIONAL_SOURCE_ROLE: &str = "thumb-switch-destinations";

#[derive(Clone, Copy, Debug)]
pub struct AdditionalTableSpec {
    pub root: u32,
    pub count: usize,
    pub code_start: u32,
    pub guard: u32,
    pub reader: u32,
    pub mov: u32,
    pub code_sha256: &'static str,
    pub source_sha256: &'static str,
}
impl AdditionalTableSpec {
    pub fn start(self) -> u32 { self.root + 4 }
    pub fn end(self) -> u32 { self.start() + self.count as u32 * 4 }
}

// Counts follow original selector comparisons; no guest index cap is added.
// At 0960da the out-of-range helper may return, so reads beyond this projection
// deliberately continue through the other owners or the original backing ROM.
pub const ADDITIONAL_TABLES: [AdditionalTableSpec; ADDITIONAL_TABLE_COUNT] = [
    AdditionalTableSpec { root: 0x960ec, count: 127, code_start: 0x960da, guard: 0x960da, reader: 0x960e2, mov: 0x960ea, code_sha256: "1da3401798e2dcb3dc32e814c1f42bca78fb28ba0b07c68335097b446dd5c481", source_sha256: "fd5a42f3ed75bc05188a750d4b5307977c48cf0bb3adcffdfc656cab8975aa3a" },
    AdditionalTableSpec { root: 0x9bacc, count: 12, code_start: 0x9baba, guard: 0x9baba, reader: 0x9bac0, mov: 0x9bac8, code_sha256: "5b5a4ee5487f10dd6949bc76a08dbe93fa3850d2b4af7641edca716d0e0b37fe", source_sha256: "375b9ee5c4a70a93139821719f10a261ba7db9ceccfe64a7f134362693e4af91" },
    AdditionalTableSpec { root: 0xac2e4, count: 5, code_start: 0xac2c4, guard: 0xac2cc, reader: 0xac2d0, mov: 0xac2d6, code_sha256: "ee6d8780350f6903451bc45902ca98e3d854d3cfc9099a8a486d3df8357cd701", source_sha256: "1fd7ecf8dc5343eb28e647413b516c70bdc7a268ffff92e19b0c22e8b2b30498" },
    AdditionalTableSpec { root: 0xce210, count: 19, code_start: 0xce1fc, guard: 0xce1fc, reader: 0xce202, mov: 0xce20a, code_sha256: "b4dbb6272a609edec70de4595e5c9a216512625549a290c39ff19553690923c3", source_sha256: "8bda2ee3ee78592db066ed81f98c265b740eb403dac217f914aed889fc07845d" },
    AdditionalTableSpec { root: 0x9efec, count: 28, code_start: 0x9efda, guard: 0x9efde, reader: 0x9efe2, mov: 0x9efea, code_sha256: "ce6b2bed9506302bc11be09268e52545c5d2d853904c48fc200e03aec9681350", source_sha256: "0582cc7d3478cbe4d1ebcd2e4a2ba2f1433f0c06079dcdf9c068f5500a2f9926" },
    AdditionalTableSpec { root: 0x9f078, count: 7, code_start: 0x9f066, guard: 0x9f068, reader: 0x9f06e, mov: 0x9f076, code_sha256: "99f790c7098ca61a01eab744839576815e3631012c72583ded91db72b3b0b8f0", source_sha256: "8752cd336fd222cf89576e352c47275fc0a01ef3ca4ebf733ac398984e5c3f3e" },
    AdditionalTableSpec { root: 0x266e0, count: 17, code_start: 0x266cc, guard: 0x266ce, reader: 0x266d8, mov: 0x266da, code_sha256: "6d722e8fbd4dfc2471ede88c6b7fdf5e39f16ae2040dcfcaa9718559bbd8493e", source_sha256: "5298d9ae1be121db1073d54b5d034749295ee19d24c61231508fe04e51bbc99d" },
    AdditionalTableSpec { root: 0x26994, count: 15, code_start: 0x2697a, guard: 0x2697e, reader: 0x2698a, mov: 0x2698c, code_sha256: "e96cb45f1cb68c3836ae0625875a7e6c00906f4c030f3e17af156ccdb8068eef", source_sha256: "517e85fb003fc0f6f521f72febc1b673cca9b310c3258dffedf8f89cdb58d624" },
    AdditionalTableSpec { root: 0x27048, count: 13, code_start: 0x2702e, guard: 0x27030, reader: 0x2703a, mov: 0x2703c, code_sha256: "2a8105037045f2a179451bf330c64d031c6571615ea30bd362a1de8b78de3adb", source_sha256: "79a280a911b0f9dc2b2235b967ba876947bf5ad2b2567dd9450d2b0bb268253a" },
    AdditionalTableSpec { root: 0x28f6c, count: 15, code_start: 0x28f56, guard: 0x28f58, reader: 0x28f62, mov: 0x28f64, code_sha256: "c5484a3ccf335ce9ca19ff5c9b0de1769b3fe2f6a10cd3f41fbe8f9200fa09de", source_sha256: "b80624145dbf22f59162bd8573f4a552f4eb1d92dc14ca6ac067e2a6f37474e5" },
    AdditionalTableSpec { root: 0x29070, count: 15, code_start: 0x29048, guard: 0x2904a, reader: 0x29056, mov: 0x29058, code_sha256: "35c8821471734acf5b0623d3f8417d41967b6792f59cc8306bc722b7e1f43fc9", source_sha256: "76f33b04960cb7eb7de638118ec6d05e419e78cca8adf6d6daff623473182237" },
    AdditionalTableSpec { root: 0xb7c24, count: 6, code_start: 0xb7c12, guard: 0xb7c14, reader: 0xb7c1e, mov: 0xb7c20, code_sha256: "73602ce8524101d46acdd260681986602d08390440ec598426e64b6804683d02", source_sha256: "2f48c9145978a524d2d554f02e3d6bb8a685d92cab32e830bde120de0639bbe0" },
    AdditionalTableSpec { root: 0xd07ec, count: 34, code_start: 0xd07dc, guard: 0xd07dc, reader: 0xd07e6, mov: 0xd07e8, code_sha256: "5c43adf9d8593097cc03bfcc30efa5a22b656c7bd3ff4d9a973bf8c2509ffa65", source_sha256: "caae8bb771b7e802bd1f0015f304c98fea684f8ce63b74ea1e4e6735016d4c32" },
    AdditionalTableSpec { root: 0xda574, count: 7, code_start: 0xda562, guard: 0xda564, reader: 0xda570, mov: 0xda572, code_sha256: "d26367adeb60cf4a3dfbc120db2626dd5c603b5bff77b86754a14121ad8a5cde", source_sha256: "0834cc3176c7af51647c8a859591747d117f7aa84fb7ac539dd9576d733869c1" },
    AdditionalTableSpec { root: 0xe3f58, count: 7, code_start: 0xe3f40, guard: 0xe3f40, reader: 0xe3f4a, mov: 0xe3f4c, code_sha256: "3c9a82e82eefbc389185eec0a269198045643360873cab5d2d6cf2898831b404", source_sha256: "547a3704765f3aeeb666f1285c05eadeae3d01b49282fb9a67c6eee1d0141663" },
    AdditionalTableSpec { root: 0x1307f4, count: 19, code_start: 0x1307e2, guard: 0x1307e4, reader: 0x1307f0, mov: 0x1307f2, code_sha256: "5af4188638d0ed6df4edc5e977e122c0b682ad666ea6f9a9ff13450f9a0ebf3a", source_sha256: "9603e440531ed52def2aaa97d9c3f9770dfc48327cf7d4bfacddef7ae2aac5d3" },
    AdditionalTableSpec { root: 0x148cc, count: 89, code_start: 0x148b6, guard: 0x148ba, reader: 0x148c6, mov: 0x148c8, code_sha256: "e879f629cf2f902f56e9954a49d404e0d0469e0a3db05bbd2839c7fd539ce1e1", source_sha256: "d11300cf1e54c9f353bcdfc3687242d7861556aef258c0300b083e954c71c70c" },
    AdditionalTableSpec { root: 0x1527c, count: 88, code_start: 0x15266, guard: 0x1526a, reader: 0x15276, mov: 0x15278, code_sha256: "1aa791185363e96db0b8f1cd0e19b1d2cf98ba2e4366027e6ffc7973e8c10fcc", source_sha256: "4e397633fc29b5d06cc339ad110f472d6b3797d534465366b4e78a18ec7a276c" },
    AdditionalTableSpec { root: 0x2abbc, count: 5, code_start: 0x2aba6, guard: 0x2aba8, reader: 0x2abb4, mov: 0x2abb6, code_sha256: "ebc5be6c4ae0ff3f7ccbd18df51021ace2f5bc05d9279858eac8990513940f7f", source_sha256: "5316a28da1a4b851f3da8100310225413f8d4cd40263e494d8fbc12ae644b385" },
    AdditionalTableSpec { root: 0x2bfb4, count: 8, code_start: 0x2bf9c, guard: 0x2bf9e, reader: 0x2bfaa, mov: 0x2bfac, code_sha256: "8fe665d95b7b611180048fd7c1a188b23ba2d655ef84f2e10e718589290d7f36", source_sha256: "613489b7a9def7eb3b821216991ee24dc5a4f4d7d55d7da5264571029696bee3" },
    AdditionalTableSpec { root: 0x5b648, count: 256, code_start: 0x5b632, guard: 0x5b634, reader: 0x5b642, mov: 0x5b644, code_sha256: "3c982dc0cebc4b136680246dd14e4950213de3cffe97d686e8e684d4640fd6ae", source_sha256: "23ea7226a8ebc2834db1f232995726e09b1a643662d4983e7b9bb721e43b7d3f" },
    AdditionalTableSpec { root: 0x1323ac, count: 15, code_start: 0x132396, guard: 0x132398, reader: 0x1323a4, mov: 0x1323a6, code_sha256: "e3b61bc54f3fd5cf28c02cceedb618fd3f066766d26d7180fd2d110fb218b444", source_sha256: "455d08e8d4f0835c0a0e68238865febc7707e9be8aa5cdbf7482eb82e7fe3696" },
    AdditionalTableSpec { root: 0xa83c8, count: 15, code_start: 0xa83b6, guard: 0xa83b6, reader: 0xa83c2, mov: 0xa83c4, code_sha256: "4d1cce1e877986f37840ca117028d3d0d33eae1cfe1b85763119a4e9508f3c22", source_sha256: "5b38334872ef1c7742fb56461db4c5796659356e8cedde12843f8ad3ead25828" },
    AdditionalTableSpec { root: 0xe0830, count: 7, code_start: 0xe0820, guard: 0xe0820, reader: 0xe082c, mov: 0xe082e, code_sha256: "758133c44ef6408852d3a7cdd1f50c2993655104152402ce8713916abac11e1a", source_sha256: "9571429b50fbb14a1d1807a475145df83e7f1cfd06cdb8d79230a126102b9168" },
    AdditionalTableSpec { root: 0xa684c, count: 8, code_start: 0xa6838, guard: 0xa6838, reader: 0xa6844, mov: 0xa6846, code_sha256: "0b79a0b5583d276d32086ce448d543d9596ebf6ce622ecc6d09ad247380c760e", source_sha256: "93ab3d93ead42d6ed5b3788c420da20526049e9e27ce87a9c6b5edd238b3242c" },
    AdditionalTableSpec { root: 0xa8228, count: 15, code_start: 0xa8208, guard: 0xa8208, reader: 0xa8214, mov: 0xa8216, code_sha256: "6eeadcd70a22c48aab98383e27b07a8f4d086cdc486f7f65b618d3e10cd4ea47", source_sha256: "23c9b8a8b9f717ab455b5ad0b1a905b92bc74f2b5e2e29eabe1256a01ef64323" },
    AdditionalTableSpec { root: 0xe5f88, count: 23, code_start: 0xe5f72, guard: 0xe5f72, reader: 0xe5f7e, mov: 0xe5f80, code_sha256: "1de07a9ddb3d78a02b98869e7694a4e96680219b23af42172b7dfe40e2df3557", source_sha256: "ac3aedb53c9642a96c21ba9323dcbb0819e13cbc0d446672a2ae12433c37fa60" },
    AdditionalTableSpec { root: 0x1335cc, count: 8, code_start: 0x1335ba, guard: 0x1335ba, reader: 0x1335c6, mov: 0x1335c8, code_sha256: "2ac6ade89f80c2c57d0eaa76372a811b0129e2e415892edf64577607c402042a", source_sha256: "ef710754b62a209d0d581883346f5dcbb605fc83692dd943508d2600023f45a3" },
    AdditionalTableSpec { root: 0x133914, count: 39, code_start: 0x133904, guard: 0x133904, reader: 0x13390e, mov: 0x133910, code_sha256: "d24f37a1078c6842244525965b8496c37d6d4b217e8c21ac1b033b75aa82b112", source_sha256: "5738f9cb4513c108d233b34978f923acbaef2ea1f5931ff0309725d1ecccd6bf" },
    AdditionalTableSpec { root: 0x133f40, count: 7, code_start: 0x133f32, guard: 0x133f32, reader: 0x133f3c, mov: 0x133f3e, code_sha256: "574b4c9e83dbdc6b4d2203317ea4ac9270f90bc77bf89d70da10287f795578c6", source_sha256: "6a106efecfa7dbc59e3591e35f83ed7682add0d779130b0283adcad77ee1fde2" },
    AdditionalTableSpec { root: 0x1343e0, count: 8, code_start: 0x1343d0, guard: 0x1343d0, reader: 0x1343da, mov: 0x1343dc, code_sha256: "3f6a31969240cbcbae7c88c80b6591f93b767b77613157bd001621e974f2fcac", source_sha256: "d157b1c4d4b07b4066751d93b37334fdaf1e0afd38ba62bf82aa44965bf7c267" },
    AdditionalTableSpec { root: 0x1357e8, count: 5, code_start: 0x1357d4, guard: 0x1357d4, reader: 0x1357de, mov: 0x1357e0, code_sha256: "c8c71afb1983252994279d553fddfcdc4b4a2219af501e2345d3ef418df2c2ab", source_sha256: "21893163878a4dba86feffd6dad5dd40fce8f3076fcde836cbed0f0950a69e56" },
    AdditionalTableSpec { root: 0xd7b7c, count: 12, code_start: 0xd7b6a, guard: 0xd7b6a, reader: 0xd7b70, mov: 0xd7b78, code_sha256: "8294cc0c7ad64711c5235ec6a5e958a23b4ad79dd12de1dad99a25765f5eab61", source_sha256: "34293fa2718c3d54a99d8cef82df1b1a1b73f030ca1209dcb32551769c86dac4" },
    AdditionalTableSpec { root: 0xd7d70, count: 14, code_start: 0xd7d60, guard: 0xd7d60, reader: 0xd7d66, mov: 0xd7d6e, code_sha256: "6cd014fae29c8952f29db73428a9ae2617ccac95095f310491eba8059c1147be", source_sha256: "3f9be23496a97567ffb922a520f3b11e64ad0e985496b0724da06fd1c5d2c7d1" },
    AdditionalTableSpec { root: 0x32e10, count: 256, code_start: 0x32dfa, guard: 0x32dfc, reader: 0x32e04, mov: 0x32e0c, code_sha256: "5659c6e1ff0e726db17bf97dd4e42bbae178a9e66937f0b3ea55e7adf1ae834f", source_sha256: "8b1cd081e8694bc98b608a3fce6b2f34449b907612e64b997d124e6103804e7f" },
    AdditionalTableSpec { root: 0x31618, count: 255, code_start: 0x31604, guard: 0x31606, reader: 0x3160e, mov: 0x31616, code_sha256: "9cdb6f0df3c24d510f0b8f1a4e3a3c13098b1efd8e6bd2ed6eeeeae2d016966c", source_sha256: "228a554873a731a0f8ef7a8cdc3d0ad168da03a93cefb3181900c8414ab580b3" },
    AdditionalTableSpec { root: 0x36320, count: 11, code_start: 0x3630c, guard: 0x3630c, reader: 0x36310, mov: 0x36318, code_sha256: "775cc9ace4cbb0c9a52e975c1cdbeece9088cbcc554bcc3b096baedd8fcd0ba9", source_sha256: "56faffee40850bd92c74f8df2abfc64972b443c3f76e8a22f16f3e768062b311" },
    AdditionalTableSpec { root: 0x3ce28, count: 10, code_start: 0x3ce0e, guard: 0x3ce0e, reader: 0x3ce14, mov: 0x3ce1c, code_sha256: "fd80655b82d49e9d2613cedeb88d81c7b5e64a993143eb3282307253847e5760", source_sha256: "417e14d37ab09cb3d6743239af11b87c04a81bf21a6915815f3ab655d87ecd9f" },
    AdditionalTableSpec { root: 0x43f90, count: 14, code_start: 0x43f76, guard: 0x43f76, reader: 0x43f7c, mov: 0x43f84, code_sha256: "daf46e2ac05cc929e0ce90ad7f17a1193dd6feb757276d626aa5fc711cb8f311", source_sha256: "be3fa4e6cb53b774842121473e5cc0946d1ad440cb73316a47afbf19b898cfbb" },
    AdditionalTableSpec { root: 0x465a8, count: 5, code_start: 0x46596, guard: 0x46598, reader: 0x4659e, mov: 0x465a6, code_sha256: "6d00f08ecee81f710d92cdd8da379b651fde673de920611aa66cce9d373c5977", source_sha256: "ff558fb16acfd53446181bbbf228f1e26f4dd9a09db6cd86c0f35b1125bd0441" },
    AdditionalTableSpec { root: 0x8d344, count: 13, code_start: 0x8d334, guard: 0x8d334, reader: 0x8d33a, mov: 0x8d342, code_sha256: "b2a1eaef97dd19cc4611c5d26878f11630b3a1996952db624b29b9de4fda4990", source_sha256: "ebaa1ee8d012be03ad48255d7543e254bc01bd4f59b5cdfdaf78ae1435b12c45" },
    AdditionalTableSpec { root: 0xd1f58, count: 65, code_start: 0xd1f42, guard: 0xd1f42, reader: 0xd1f48, mov: 0xd1f50, code_sha256: "0d562d7b02eca47a98cba65601e5abfed36b11beaed0af6ee8ff9f76f8a7e2b6", source_sha256: "88d3b524ccdc52a5ff30653f7f85b1373d5b65b38f1507e47bc6cd29b9dcda51" },
    // The out-of-domain helper at 0bbfd4 may return; retain original fallback reads.
    AdditionalTableSpec { root: 0x4e038, count: 7, code_start: 0x4e026, guard: 0x4e026, reader: 0x4e02c, mov: 0x4e034, code_sha256: "58d59b2dd8d19bb30e7ee20caaa28821c8b8be2d6e423e71b1cdb3c87ed0d23f", source_sha256: "14492e0365736815cee36e17289205bbcccd3f1a700d54cb27a1e0259b929edc" },
    AdditionalTableSpec { root: 0x8dfd0, count: 14, code_start: 0x8dfbc, guard: 0x8dfbc, reader: 0x8dfc2, mov: 0x8dfca, code_sha256: "19d563ac0da9c8e6c283a2121ee8bffe67da3f5ce6425d2357290901b69b6d73", source_sha256: "8d9ae0552b0d0f60119d9c5e047fe09a6be681c08d48151608b1c071ebb6ccb1" },
    AdditionalTableSpec { root: 0xbbffc, count: 37, code_start: 0xbbfd0, guard: 0xbbfd0, reader: 0xbbfd8, mov: 0xbbfe0, code_sha256: "764ea1207578b3b52e470ef19836eba1e3e53091728113c4f0fbe0b0a1000440", source_sha256: "b0fe306c4eeb85923cc49bde70523ec62e93f1d15726f623eb129cd64f8df7ea" },
    AdditionalTableSpec { root: 0xcd950, count: 23, code_start: 0xcd940, guard: 0xcd940, reader: 0xcd944, mov: 0xcd94c, code_sha256: "455d92b9f63640eda9c7e92f32be9e143d86d6f9d1809bbfdf378acd6cde5a36", source_sha256: "a11a6ce18489b156059bf83fe5768d1ee5f598c28908d5da6c8ce49538189309" },
    AdditionalTableSpec { root: 0x125c88, count: 8, code_start: 0x125c78, guard: 0x125c78, reader: 0x125c7e, mov: 0x125c86, code_sha256: "2ee5f9187db66e5d62db01a18a981c7ebc1dabb83dacbaec9d7a0f81e88572ad", source_sha256: "25fee10c047dcfd1cb533c836c1a0f83702040b77254b64784d3bb6045ae9742" },
    AdditionalTableSpec { root: 0x2a7b0, count: 11, code_start: 0x2a79a, guard: 0x2a79c, reader: 0x2a7a0, mov: 0x2a7a8, code_sha256: "969664e21ea18268c37749c2fdbb2344a871a197dfc9af9988243a059885e396", source_sha256: "053cee09d503cbbdbd14008f795e340443f838d51b1cfb05aa0104eabc1cfb70" },
    AdditionalTableSpec { root: 0x125fe4, count: 12, code_start: 0x125fd0, guard: 0x125fd4, reader: 0x125fd8, mov: 0x125fe0, code_sha256: "aab18da1d78a8af88cdd98f662d4182fa36a0a7bef9c9b1fbfa5d86a9007fdb0", source_sha256: "926f076935f206b158f82b216ec66ca605afdf6e5cb87a44d69829804302909c" },
    AdditionalTableSpec { root: 0x1279b8, count: 7, code_start: 0x12798a, guard: 0x12798a, reader: 0x127990, mov: 0x127998, code_sha256: "d4ed6bdee7c2abeeb8a5ae5424bd9fadd009a5d0608cc42e079557a387e18adf", source_sha256: "fc3a42742eebc2534bc0732a900a841e4ca09cfc9bf03e62cf58a97993d94f16" },
    AdditionalTableSpec {
        root: 0x13ad28, count: 6, code_start: 0x13ad10, guard: 0x13ad10,
        reader: 0x13ad1c, mov: 0x13ad1e,
        code_sha256: "4df3020d75ca7e6ffa3f611afb237e27b15956c1423e002b6abaec8a98c73202",
        source_sha256: "e68b3e2802ce95f9b26a30fe4e8276f4ff7249ee8fffd1c86c153c89f0263017",
    },
    AdditionalTableSpec {
        root: 0x13ba38, count: 5, code_start: 0x13ba22, guard: 0x13ba22,
        reader: 0x13ba2c, mov: 0x13ba2e,
        code_sha256: "d1a39392250cf0f59b6031d9b6476fbaa35b00684cab76a0c9be1b1565be1a93",
        source_sha256: "e761bed0b7f27d0f831c3ce3ccc585751a804c87830f5fa12d307e555958bc71",
    },
    AdditionalTableSpec {
        root: 0x386b0, count: 5, code_start: 0x38696, guard: 0x38698,
        reader: 0x386a4, mov: 0x386a6,
        code_sha256: "6b075ee1a4a01b0e0b9b5260c3ec520d8fd318bc99ae9408fd8de1a63aee82f3",
        source_sha256: "66eaa68630ec19b1f97cb4f51bfca0cca228e289c203f103e0f4c908d234591d",
    },
    AdditionalTableSpec {
        root: 0x096e54, count: 7, code_start: 0x096e42, guard: 0x096e42,
        reader: 0x096e48, mov: 0x096e50,
        code_sha256: "26b5a299cdce9790b0b2298bf5d60ac2767f244aec27a776956c81fb57d23db2",
        source_sha256: "b6b0e5b1043ef0b02f02525d9e402e296375976a6838acfcf2f944386c8102a7",
    },
    AdditionalTableSpec {
        root: 0x097644, count: 11, code_start: 0x097630, guard: 0x097632,
        reader: 0x09763a, mov: 0x097642,
        code_sha256: "125aed9f652116fd437a6d17ddec07bb474a9264504c7efabc8cc40cb649e226",
        source_sha256: "a8a3ded6d5ef15cce36209b4a2e58a08e1616dc7e880a9f796388a7744333b18",
    },
];

const CACHED_OBJECT_TABLES: [AdditionalTableSpec; 2] = [
    AdditionalTableSpec {
        root: 0x0089d8, count: 11, code_start: 0x0089c4, guard: 0x0089c4,
        reader: 0x0089ca, mov: 0x0089d2,
        code_sha256: "6201f77e0713fbefa6c2cc45f9aff6515c219c96efe02b76a6a16e6e20af8b97",
        source_sha256: "48281e2051e39651e47e30cd3590f775ac615c6ab29619535d42d1b755fe8f28",
    },
    AdditionalTableSpec {
        root: 0x009560, count: 10, code_start: 0x009532, guard: 0x009546,
        reader: 0x00954c, mov: 0x009554,
        code_sha256: "81eb2a5a25027e4bfdffbb4184d710a68807213dadf49497589cc92fbdda8c69",
        source_sha256: "522c09f62603832e3aa7848f375a19f033f5a98d1d287adc8e5739da9589e0ec",
    },
];

const INDEXED_EXCHANGE_TABLES: [AdditionalTableSpec; 2] = [
    AdditionalTableSpec {
        root: 0x0bb5a8, count: 13, code_start: 0x0bb594, guard: 0x0bb59a,
        reader: 0x0bb59e, mov: 0x0bb5a6,
        code_sha256: "197082f65db72e44dc6f136436369a0b3515267b3c887e3ff54cb2782c25f45a",
        source_sha256: "d836079ef0bdc27240d1ded73ee80d9e99ea31605c9dab26af04129973179838",
    },
    AdditionalTableSpec {
        root: 0x0c3080, count: 5, code_start: 0x0c306e, guard: 0x0c3070,
        reader: 0x0c3076, mov: 0x0c307e,
        code_sha256: "e6f6bcfb935b7f796dabda420bdf11ebd6bfdc4c0411e8268b82f15c5e7931a4",
        source_sha256: "174191e84754b314d8867b119c8affd5c326be7001b2f8ed71a09711aa714722",
    },
];

const FIVE_BYTE_LIST_TABLES: [AdditionalTableSpec; 1] = [AdditionalTableSpec {
    root: 0x14c574, count: 5, code_start: 0x14c560, guard: 0x14c564,
    reader: 0x14c56c, mov: 0x14c572,
    code_sha256: "7846e24df016cb34720f49d3ea4b7394ab7f7d0dcade3c89dc096c7d43efa2ec",
    source_sha256: "f48521eee7fdec77cf13e582c85adf7395f2c2f8c919d50e2ddb139e83c76fac",
}];

const BACKGROUND_VARIANT_STATE_TABLE: AdditionalTableSpec = AdditionalTableSpec {
    root: 0x0209a0,
    count: 14,
    code_start: 0x02097c,
    guard: 0x02098c,
    reader: 0x020992,
    mov: 0x02099a,
    code_sha256: "0df1b059f0ad5e88765a397df4354acb020d177fd77c84c806f8509daac6cc61",
    source_sha256: "45064df4daee4c578eb674782e84c3c210551c9d13fa9323a7dbaf05655b29a1",
};

pub(super) fn additional_table_specs() -> impl Iterator<Item = &'static AdditionalTableSpec> {
    ADDITIONAL_TABLES.iter().chain(&CACHED_OBJECT_TABLES).chain(&INDEXED_EXCHANGE_TABLES)
        .chain(&FIVE_BYTE_LIST_TABLES)
        .chain(std::iter::once(&BACKGROUND_VARIANT_STATE_TABLE))
}

/// Even target address used by the original Thumb `MOV pc` instruction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThumbDestination {
    pub address: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwitchTable {
    pub root_offset: u32,
    pub table_offset: u32,
    pub destinations: Vec<ThumbDestination>,
}
impl SwitchTable {
    pub fn source_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.destinations.len() * 4);
        out.extend_from_slice(&(0x0800_0000u32 + self.table_offset).to_le_bytes());
        out.extend_from_slice(&self.destination_bytes());
        out
    }

    pub fn destination_bytes(&self) -> Vec<u8> {
        self.destinations.iter().flat_map(|destination| destination.address.to_le_bytes()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThumbSwitchTables {
    pub tables: Vec<SwitchTable>,
    pub additional_tables: Vec<SwitchTable>,
}
impl ThumbSwitchTables {
    pub fn validate(&self) -> Result<(), String> {
        if self.tables.len() != TABLE_COUNT { return Err("Invalid EUR Thumb switch-table count".into()); }
        for (table, spec) in self.tables.iter().zip(TABLES) {
            if table.root_offset != spec.root || table.table_offset != spec.root + 4
                || table.destinations.len() != spec.count
                || table.destinations.iter().any(|d| d.address & 1 != 0
                    || !(0x0800_0000..0x0820_0000).contains(&d.address)) {
                return Err("Invalid EUR Thumb switch-table source, target or order".into());
            }
        }
        if ![
            PREVIOUS_ADDITIONAL_TABLE_COUNT,
            FIRAGA_ADDITIONAL_TABLE_COUNT,
            WHITE_FLOWERS_ADDITIONAL_TABLE_COUNT,
            WHITE_FLOWERS_RESIDUAL_TABLE_COUNT,
            WHITE_FLOWERS_MIXED_TABLE_COUNT,
            PRE_TITLE_ADDITIONAL_TABLE_COUNT,
            TITLE_ADDITIONAL_TABLE_COUNT,
            WORLD_NODE_ADDITIONAL_TABLE_COUNT,
            ADDITIONAL_TABLE_COUNT,
            CACHED_OBJECT_ADDITIONAL_TABLE_COUNT,
            INDEXED_EXCHANGE_ADDITIONAL_TABLE_COUNT,
            FIVE_BYTE_LIST_ADDITIONAL_TABLE_COUNT,
            BACKGROUND_VARIANT_STATE_TABLE_COUNT,
        ].contains(&self.additional_tables.len()) {
            return Err("Invalid additional EUR Thumb switch-table count".into());
        }
        for (table, spec) in self.additional_tables.iter().zip(additional_table_specs()) {
            if table.root_offset != spec.root || table.table_offset != spec.start()
                || table.destinations.len() != spec.count
                || table.destinations.iter().any(|d| d.address & 1 != 0
                    || !(0x0800_0000..0x0820_0000).contains(&d.address)) {
                return Err("Invalid additional EUR Thumb switch-table source, target or order".into());
            }
        }
        Ok(())
    }

    /// Returns a byte from a root or target; guest guards and control flow stay original.
    pub fn byte_at(&self, offset: u32) -> Option<(u8, usize, bool)> {
        for (index, table) in self.tables.iter().enumerate() {
            let Some(relative) = offset.checked_sub(table.root_offset) else { continue };
            let relative = relative as usize;
            if relative < 4 {
                return Some(((0x0800_0000 + table.table_offset).to_le_bytes()[relative], index, true));
            }
            let target = (relative - 4) / 4;
            if let Some(destination) = table.destinations.get(target) {
                return Some((destination.address.to_le_bytes()[(relative - 4) % 4], index, false));
            }
        }
        for (index, table) in self.additional_tables.iter().enumerate() {
            let Some(relative) = offset.checked_sub(table.table_offset) else { continue };
            let relative = relative as usize;
            if let Some(destination) = table.destinations.get(relative / 4) {
                return Some((destination.address.to_le_bytes()[relative % 4], TABLE_COUNT + index, false));
            }
        }
        None
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        let layout = match self.additional_tables.len() {
            BACKGROUND_VARIANT_STATE_TABLE_COUNT => SourceLayout::BackgroundVariantState,
            FIVE_BYTE_LIST_ADDITIONAL_TABLE_COUNT => SourceLayout::FiveByteList,
            INDEXED_EXCHANGE_ADDITIONAL_TABLE_COUNT => SourceLayout::IndexedExchange,
            CACHED_OBJECT_ADDITIONAL_TABLE_COUNT => SourceLayout::CachedObject,
            ADDITIONAL_TABLE_COUNT => SourceLayout::HelperFieldDispatch,
            WORLD_NODE_ADDITIONAL_TABLE_COUNT => SourceLayout::WorldNodeRenderer,
            TITLE_ADDITIONAL_TABLE_COUNT => SourceLayout::TitleFrontend,
            _ => SourceLayout::Legacy,
        };
        if self != &Catalog::from_rom_with_layout(rom, &[], layout)?.decoded {
            return Err("Thumb switch tables differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ThumbSwitchTables,
    pub sources: Vec<SourceRange>,
}

pub(super) enum SourceLayout {
    Legacy,
    TitleFrontend,
    WorldNodeRenderer,
    HelperFieldDispatch,
    CachedObject,
    IndexedExchange,
    FiveByteList,
    BackgroundVariantState,
}

impl Catalog {
    pub fn from_rom(rom: &[u8], previous: &[Resource]) -> Result<Self, String> {
        Self::from_rom_with_layout(rom, previous, SourceLayout::HelperFieldDispatch)
    }

    pub(super) fn from_rom_with_layout(
        rom: &[u8], previous: &[Resource], layout: SourceLayout,
    ) -> Result<Self, String> {
        if rom.len() != 0x0100_0000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Thumb switch tables require exact EUR ROM; USA pending".into());
        }
        let half = |at: u32| u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
        let word = |at: u32| u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
        let prior = PriorSources::new(previous)?;
        let mut tables = Vec::with_capacity(TABLE_COUNT);
        let additional_count = match layout {
            SourceLayout::Legacy => PRE_TITLE_ADDITIONAL_TABLE_COUNT,
            SourceLayout::TitleFrontend => TITLE_ADDITIONAL_TABLE_COUNT,
            SourceLayout::WorldNodeRenderer => WORLD_NODE_ADDITIONAL_TABLE_COUNT,
            SourceLayout::HelperFieldDispatch => ADDITIONAL_TABLE_COUNT,
            SourceLayout::CachedObject => CACHED_OBJECT_ADDITIONAL_TABLE_COUNT,
            SourceLayout::IndexedExchange => INDEXED_EXCHANGE_ADDITIONAL_TABLE_COUNT,
            SourceLayout::FiveByteList => FIVE_BYTE_LIST_ADDITIONAL_TABLE_COUNT,
            SourceLayout::BackgroundVariantState => BACKGROUND_VARIANT_STATE_TABLE_COUNT,
        };
        let mut sources = Vec::with_capacity(TABLE_COUNT + additional_count);
        for spec in TABLES {
            let code_start = spec.guard.min(spec.reader) as usize;
            if ffta_rom::digest(&rom[code_start..spec.mov as usize + 2]) != spec.code_sha256
                || half(spec.mov) != 0x4687 || word(spec.root) != 0x0800_0000 + spec.root + 4 {
                return Err("Thumb switch reader or root fingerprint mismatch".into());
            }
            if spec.special {
                // This reader keeps an out-of-range selector in its original compare loop.
                if half(spec.reader) != 0x4e04 || half(spec.guard) != 0x2d2f
                    || half(spec.guard + 2) != 0xd8fd || half(spec.guard + 4) != 0x00a8
                    || half(spec.guard + 6) != 0x1980 || half(spec.guard + 8) != 0x6800 {
                    return Err("Special Thumb switch guard mismatch".into());
                }
            } else {
                let compare = half(spec.guard);
                let branch = half(spec.guard + 2);
                let signed_displacement = (branch as u8 as i8 as i32) * 2;
                let target = (spec.guard as i64 + 6 + i64::from(signed_displacement)) as u32;
                let ldr = half(spec.reader + 2);
                if compare & 0xff00 != 0x2800 || usize::from(compare & 255) + 1 != spec.count
                    || !((branch & 0xff00 == 0xd900 && target == spec.reader)
                        || (branch & 0xff00 == 0xd800 && target >= spec.end()))
                    || half(spec.reader) != 0x0080 || ldr & 0xff00 != 0x4900
                    || ((spec.reader + 6) & !3) + u32::from(ldr & 255) * 4 != spec.root
                    || half(spec.reader + 4) != 0x1840 || half(spec.reader + 6) != 0x6800 {
                    return Err("Thumb switch guard, base or load mismatch".into());
                }
            }
            let raw = &rom[spec.root as usize..spec.end() as usize];
            if ffta_rom::digest(raw) != spec.source_sha256 || prior.contains(spec.root, raw.len())? {
                return Err("Thumb switch source fingerprint or owner mismatch".into());
            }
            let destinations = (0..spec.count).map(|i| ThumbDestination {
                address: word(spec.root + 4 + i as u32 * 4),
            }).collect();
            let table = SwitchTable { root_offset: spec.root, table_offset: spec.root + 4, destinations };
            if table.source_bytes() != raw { return Err("Thumb switch reconstruction mismatch".into()); }
            sources.push(SourceRange { offset: spec.root, length: raw.len() as u32,
                sha256: spec.source_sha256.into(), role: SOURCE_ROLE.into() });
            tables.push(table);
        }
        let mut additional_tables = Vec::with_capacity(additional_count);
        for spec in additional_table_specs().take(additional_count) {
            if ffta_rom::digest(&rom[spec.code_start as usize..spec.mov as usize + 2]) != spec.code_sha256
                || half(spec.mov) != 0x4687 || word(spec.root) != 0x0800_0000 + spec.start() {
                return Err("Additional Thumb switch reader or root fingerprint mismatch".into());
            }
            let raw = &rom[spec.start() as usize..spec.end() as usize];
            if ffta_rom::digest(raw) != spec.source_sha256 || prior.contains(spec.start(), raw.len())?
                || TABLES.iter().any(|old| spec.start() < old.end() && old.root < spec.end()) {
                return Err("Additional Thumb switch source fingerprint or owner mismatch".into());
            }
            let destinations = (0..spec.count).map(|i| ThumbDestination {
                address: word(spec.start() + i as u32 * 4),
            }).collect();
            let table = SwitchTable { root_offset: spec.root, table_offset: spec.start(), destinations };
            if table.destination_bytes() != raw { return Err("Additional Thumb switch reconstruction mismatch".into()); }
            sources.push(SourceRange { offset: spec.start(), length: raw.len() as u32,
                sha256: spec.source_sha256.into(), role: ADDITIONAL_SOURCE_ROLE.into() });
            additional_tables.push(table);
        }
        let decoded = ThumbSwitchTables { tables, additional_tables };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}
