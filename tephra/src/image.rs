use crate::buffer::Buffer;
use crate::context::Context;
use crate::downcast::Downcast;
use derive_builder::Builder;
use slotmap::new_key_type;
new_key_type!(
    pub struct ImageHandle;
);
//use renderpass::{Pass, Renderpass};
#[derive(Debug, Copy, Clone)]
pub enum ImageLayout {
    Color,
    Depth,
}
#[derive(Debug, Copy, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

pub trait CreateImage {}

pub trait ImageApi {
    fn allocate_image(&self, desc: ImageDesc) -> ImageHandle;
    fn from_buffer(&self, buffer: Buffer<u8>) -> ImageHandle;
    fn desc(&self, handle: ImageHandle) -> ImageDesc;
    fn copy_image(&self, src: ImageHandle, dst: ImageHandle);
}

#[derive(Copy, Clone)]
pub struct Image {
    pub handle: ImageHandle,
}

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct ImageDesc {
    pub resolution: Resolution,
    pub layout: ImageLayout,
    pub format: Format,
}

impl Image {
    pub fn allocate(ctx: &Context, desc: ImageDesc) -> Image {
        let handle = ctx.allocate_image(desc);
        Image { handle }
    }
}

pub struct RenderTargetInfo<'a> {
    pub image_views: Vec<&'a Image>,
}

pub trait RenderTarget<'a> {
    fn render_target(&self) -> RenderTargetInfo;
}

pub trait CreateFramebuffer {
    fn new(&self, render_target: &RenderTargetInfo) -> Self;
}

pub trait FramebufferApi: Downcast {}
impl_downcast!(FramebufferApi);

pub struct Framebuffer<T>
where
    for<'a> T: RenderTarget<'a>,
{
    pub render_target: T,
    pub data: Box<dyn FramebufferApi>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Format(pub(crate) i32);
impl Format {
    pub fn from_raw(x: i32) -> Self {
        Format(x)
    }
    pub fn as_raw(self) -> i32 {
        self.0
    }
}
impl Format {
    pub const UNDEFINED: Self = Format(0);
    pub const R4G4_UNORM_PACK8: Self = Format(1);
    pub const R4G4B4A4_UNORM_PACK16: Self = Format(2);
    pub const B4G4R4A4_UNORM_PACK16: Self = Format(3);
    pub const R5G6B5_UNORM_PACK16: Self = Format(4);
    pub const B5G6R5_UNORM_PACK16: Self = Format(5);
    pub const R5G5B5A1_UNORM_PACK16: Self = Format(6);
    pub const B5G5R5A1_UNORM_PACK16: Self = Format(7);
    pub const A1R5G5B5_UNORM_PACK16: Self = Format(8);
    pub const R8_UNORM: Self = Format(9);
    pub const R8_SNORM: Self = Format(10);
    pub const R8_USCALED: Self = Format(11);
    pub const R8_SSCALED: Self = Format(12);
    pub const R8_UINT: Self = Format(13);
    pub const R8_SINT: Self = Format(14);
    pub const R8_SRGB: Self = Format(15);
    pub const R8G8_UNORM: Self = Format(16);
    pub const R8G8_SNORM: Self = Format(17);
    pub const R8G8_USCALED: Self = Format(18);
    pub const R8G8_SSCALED: Self = Format(19);
    pub const R8G8_UINT: Self = Format(20);
    pub const R8G8_SINT: Self = Format(21);
    pub const R8G8_SRGB: Self = Format(22);
    pub const R8G8B8_UNORM: Self = Format(23);
    pub const R8G8B8_SNORM: Self = Format(24);
    pub const R8G8B8_USCALED: Self = Format(25);
    pub const R8G8B8_SSCALED: Self = Format(26);
    pub const R8G8B8_UINT: Self = Format(27);
    pub const R8G8B8_SINT: Self = Format(28);
    pub const R8G8B8_SRGB: Self = Format(29);
    pub const B8G8R8_UNORM: Self = Format(30);
    pub const B8G8R8_SNORM: Self = Format(31);
    pub const B8G8R8_USCALED: Self = Format(32);
    pub const B8G8R8_SSCALED: Self = Format(33);
    pub const B8G8R8_UINT: Self = Format(34);
    pub const B8G8R8_SINT: Self = Format(35);
    pub const B8G8R8_SRGB: Self = Format(36);
    pub const R8G8B8A8_UNORM: Self = Format(37);
    pub const R8G8B8A8_SNORM: Self = Format(38);
    pub const R8G8B8A8_USCALED: Self = Format(39);
    pub const R8G8B8A8_SSCALED: Self = Format(40);
    pub const R8G8B8A8_UINT: Self = Format(41);
    pub const R8G8B8A8_SINT: Self = Format(42);
    pub const R8G8B8A8_SRGB: Self = Format(43);
    pub const B8G8R8A8_UNORM: Self = Format(44);
    pub const B8G8R8A8_SNORM: Self = Format(45);
    pub const B8G8R8A8_USCALED: Self = Format(46);
    pub const B8G8R8A8_SSCALED: Self = Format(47);
    pub const B8G8R8A8_UINT: Self = Format(48);
    pub const B8G8R8A8_SINT: Self = Format(49);
    pub const B8G8R8A8_SRGB: Self = Format(50);
    pub const A8B8G8R8_UNORM_PACK32: Self = Format(51);
    pub const A8B8G8R8_SNORM_PACK32: Self = Format(52);
    pub const A8B8G8R8_USCALED_PACK32: Self = Format(53);
    pub const A8B8G8R8_SSCALED_PACK32: Self = Format(54);
    pub const A8B8G8R8_UINT_PACK32: Self = Format(55);
    pub const A8B8G8R8_SINT_PACK32: Self = Format(56);
    pub const A8B8G8R8_SRGB_PACK32: Self = Format(57);
    pub const A2R10G10B10_UNORM_PACK32: Self = Format(58);
    pub const A2R10G10B10_SNORM_PACK32: Self = Format(59);
    pub const A2R10G10B10_USCALED_PACK32: Self = Format(60);
    pub const A2R10G10B10_SSCALED_PACK32: Self = Format(61);
    pub const A2R10G10B10_UINT_PACK32: Self = Format(62);
    pub const A2R10G10B10_SINT_PACK32: Self = Format(63);
    pub const A2B10G10R10_UNORM_PACK32: Self = Format(64);
    pub const A2B10G10R10_SNORM_PACK32: Self = Format(65);
    pub const A2B10G10R10_USCALED_PACK32: Self = Format(66);
    pub const A2B10G10R10_SSCALED_PACK32: Self = Format(67);
    pub const A2B10G10R10_UINT_PACK32: Self = Format(68);
    pub const A2B10G10R10_SINT_PACK32: Self = Format(69);
    pub const R16_UNORM: Self = Format(70);
    pub const R16_SNORM: Self = Format(71);
    pub const R16_USCALED: Self = Format(72);
    pub const R16_SSCALED: Self = Format(73);
    pub const R16_UINT: Self = Format(74);
    pub const R16_SINT: Self = Format(75);
    pub const R16_SFLOAT: Self = Format(76);
    pub const R16G16_UNORM: Self = Format(77);
    pub const R16G16_SNORM: Self = Format(78);
    pub const R16G16_USCALED: Self = Format(79);
    pub const R16G16_SSCALED: Self = Format(80);
    pub const R16G16_UINT: Self = Format(81);
    pub const R16G16_SINT: Self = Format(82);
    pub const R16G16_SFLOAT: Self = Format(83);
    pub const R16G16B16_UNORM: Self = Format(84);
    pub const R16G16B16_SNORM: Self = Format(85);
    pub const R16G16B16_USCALED: Self = Format(86);
    pub const R16G16B16_SSCALED: Self = Format(87);
    pub const R16G16B16_UINT: Self = Format(88);
    pub const R16G16B16_SINT: Self = Format(89);
    pub const R16G16B16_SFLOAT: Self = Format(90);
    pub const R16G16B16A16_UNORM: Self = Format(91);
    pub const R16G16B16A16_SNORM: Self = Format(92);
    pub const R16G16B16A16_USCALED: Self = Format(93);
    pub const R16G16B16A16_SSCALED: Self = Format(94);
    pub const R16G16B16A16_UINT: Self = Format(95);
    pub const R16G16B16A16_SINT: Self = Format(96);
    pub const R16G16B16A16_SFLOAT: Self = Format(97);
    pub const R32_UINT: Self = Format(98);
    pub const R32_SINT: Self = Format(99);
    pub const R32_SFLOAT: Self = Format(100);
    pub const R32G32_UINT: Self = Format(101);
    pub const R32G32_SINT: Self = Format(102);
    pub const R32G32_SFLOAT: Self = Format(103);
    pub const R32G32B32_UINT: Self = Format(104);
    pub const R32G32B32_SINT: Self = Format(105);
    pub const R32G32B32_SFLOAT: Self = Format(106);
    pub const R32G32B32A32_UINT: Self = Format(107);
    pub const R32G32B32A32_SINT: Self = Format(108);
    pub const R32G32B32A32_SFLOAT: Self = Format(109);
    pub const R64_UINT: Self = Format(110);
    pub const R64_SINT: Self = Format(111);
    pub const R64_SFLOAT: Self = Format(112);
    pub const R64G64_UINT: Self = Format(113);
    pub const R64G64_SINT: Self = Format(114);
    pub const R64G64_SFLOAT: Self = Format(115);
    pub const R64G64B64_UINT: Self = Format(116);
    pub const R64G64B64_SINT: Self = Format(117);
    pub const R64G64B64_SFLOAT: Self = Format(118);
    pub const R64G64B64A64_UINT: Self = Format(119);
    pub const R64G64B64A64_SINT: Self = Format(120);
    pub const R64G64B64A64_SFLOAT: Self = Format(121);
    pub const B10G11R11_UFLOAT_PACK32: Self = Format(122);
    pub const E5B9G9R9_UFLOAT_PACK32: Self = Format(123);
    pub const D16_UNORM: Self = Format(124);
    pub const X8_D24_UNORM_PACK32: Self = Format(125);
    pub const D32_SFLOAT: Self = Format(126);
    pub const S8_UINT: Self = Format(127);
    pub const D16_UNORM_S8_UINT: Self = Format(128);
    pub const D24_UNORM_S8_UINT: Self = Format(129);
    pub const D32_SFLOAT_S8_UINT: Self = Format(130);
    pub const BC1_RGB_UNORM_BLOCK: Self = Format(131);
    pub const BC1_RGB_SRGB_BLOCK: Self = Format(132);
    pub const BC1_RGBA_UNORM_BLOCK: Self = Format(133);
    pub const BC1_RGBA_SRGB_BLOCK: Self = Format(134);
    pub const BC2_UNORM_BLOCK: Self = Format(135);
    pub const BC2_SRGB_BLOCK: Self = Format(136);
    pub const BC3_UNORM_BLOCK: Self = Format(137);
    pub const BC3_SRGB_BLOCK: Self = Format(138);
    pub const BC4_UNORM_BLOCK: Self = Format(139);
    pub const BC4_SNORM_BLOCK: Self = Format(140);
    pub const BC5_UNORM_BLOCK: Self = Format(141);
    pub const BC5_SNORM_BLOCK: Self = Format(142);
    pub const BC6H_UFLOAT_BLOCK: Self = Format(143);
    pub const BC6H_SFLOAT_BLOCK: Self = Format(144);
    pub const BC7_UNORM_BLOCK: Self = Format(145);
    pub const BC7_SRGB_BLOCK: Self = Format(146);
    pub const ETC2_R8G8B8_UNORM_BLOCK: Self = Format(147);
    pub const ETC2_R8G8B8_SRGB_BLOCK: Self = Format(148);
    pub const ETC2_R8G8B8A1_UNORM_BLOCK: Self = Format(149);
    pub const ETC2_R8G8B8A1_SRGB_BLOCK: Self = Format(150);
    pub const ETC2_R8G8B8A8_UNORM_BLOCK: Self = Format(151);
    pub const ETC2_R8G8B8A8_SRGB_BLOCK: Self = Format(152);
    pub const EAC_R11_UNORM_BLOCK: Self = Format(153);
    pub const EAC_R11_SNORM_BLOCK: Self = Format(154);
    pub const EAC_R11G11_UNORM_BLOCK: Self = Format(155);
    pub const EAC_R11G11_SNORM_BLOCK: Self = Format(156);
    pub const ASTC_4X4_UNORM_BLOCK: Self = Format(157);
    pub const ASTC_4X4_SRGB_BLOCK: Self = Format(158);
    pub const ASTC_5X4_UNORM_BLOCK: Self = Format(159);
    pub const ASTC_5X4_SRGB_BLOCK: Self = Format(160);
    pub const ASTC_5X5_UNORM_BLOCK: Self = Format(161);
    pub const ASTC_5X5_SRGB_BLOCK: Self = Format(162);
    pub const ASTC_6X5_UNORM_BLOCK: Self = Format(163);
    pub const ASTC_6X5_SRGB_BLOCK: Self = Format(164);
    pub const ASTC_6X6_UNORM_BLOCK: Self = Format(165);
    pub const ASTC_6X6_SRGB_BLOCK: Self = Format(166);
    pub const ASTC_8X5_UNORM_BLOCK: Self = Format(167);
    pub const ASTC_8X5_SRGB_BLOCK: Self = Format(168);
    pub const ASTC_8X6_UNORM_BLOCK: Self = Format(169);
    pub const ASTC_8X6_SRGB_BLOCK: Self = Format(170);
    pub const ASTC_8X8_UNORM_BLOCK: Self = Format(171);
    pub const ASTC_8X8_SRGB_BLOCK: Self = Format(172);
    pub const ASTC_10X5_UNORM_BLOCK: Self = Format(173);
    pub const ASTC_10X5_SRGB_BLOCK: Self = Format(174);
    pub const ASTC_10X6_UNORM_BLOCK: Self = Format(175);
    pub const ASTC_10X6_SRGB_BLOCK: Self = Format(176);
    pub const ASTC_10X8_UNORM_BLOCK: Self = Format(177);
    pub const ASTC_10X8_SRGB_BLOCK: Self = Format(178);
    pub const ASTC_10X10_UNORM_BLOCK: Self = Format(179);
    pub const ASTC_10X10_SRGB_BLOCK: Self = Format(180);
    pub const ASTC_12X10_UNORM_BLOCK: Self = Format(181);
    pub const ASTC_12X10_SRGB_BLOCK: Self = Format(182);
    pub const ASTC_12X12_UNORM_BLOCK: Self = Format(183);
    pub const ASTC_12X12_SRGB_BLOCK: Self = Format(184);
}
