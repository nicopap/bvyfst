#![allow(unused)]
use std::num::{NonZeroU16, NonZeroU32};

use ::bevy::render::render_resource::Face;
use bevy::prelude as bevy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::Archived;

mod material_flags {

    use super::*;

    #[rustfmt::skip]
    mod start_bit {
        pub(super) const ALPHA: u16 = 4;
        pub(super) const FACE: u16  = 3 + ALPHA;
        pub(super) const _END: u16  = 2 + FACE;
    }

    #[derive(Clone, Copy, Archive, Deserialize, Serialize)]
    pub struct MaterialFlags(u16);
    impl ArchivedMaterialFlags {
        pub const fn flip_normal_map_y(&self) -> bool {
            self.0 & MaterialFlagsInner::FLIP_NORMAL_MAP_Y.bits() != 0
        }
        pub const fn double_sided(&self) -> bool {
            self.0 & MaterialFlagsInner::DOUBLE_SIDED.bits() != 0
        }
        pub const fn unlit(&self) -> bool {
            self.0 & MaterialFlagsInner::UNLIT.bits() != 0
        }
        pub const fn fog_enabled(&self) -> bool {
            self.0 & MaterialFlagsInner::FOG_ENABLED.bits() != 0
        }
        pub const fn alpha_mode(&self, mask_threshold: f32) -> bevy::AlphaMode {
            let mask = 0b111 << start_bit::ALPHA;
            match (self.0 & mask) >> start_bit::ALPHA {
                0b000 => bevy::AlphaMode::Opaque,
                0b001 => bevy::AlphaMode::Mask(mask_threshold),
                0b010 => bevy::AlphaMode::Blend,
                0b011 => bevy::AlphaMode::Premultiplied,
                0b100 => bevy::AlphaMode::Add,
                0b101 => bevy::AlphaMode::Multiply,
                _ => panic!("MaterialFlags was malformed"),
            }
        }
        pub const fn cull_mode(&self) -> Option<Face> {
            let mask = 0b11 << start_bit::FACE;
            match (self.0 & mask) >> start_bit::FACE {
                0b00 => None,
                0b01 => Some(Face::Front),
                0b11 => Some(Face::Back),
                _ => panic!("MaterialFlags was malformed"),
            }
        }
    }

    bitflags::bitflags! {
        #[repr(transparent)]
        struct MaterialFlagsInner: u16 {
            const FLIP_NORMAL_MAP_Y = 0b0001;
            const DOUBLE_SIDED      = 0b0010;
            const UNLIT             = 0b0100;
            const FOG_ENABLED       = 0b1000;

            // Alpha Mode
            const OPAQUE            = 0b000 << start_bit::ALPHA;
            const MASK              = 0b001 << start_bit::ALPHA;
            const BLEND             = 0b010 << start_bit::ALPHA;
            const PREMULTIPLIED     = 0b011 << start_bit::ALPHA;
            const ADD               = 0b100 << start_bit::ALPHA;
            const MULTIPLY          = 0b101 << start_bit::ALPHA;

            // Option<Face>
            const NONE              = 0b00 << start_bit::FACE;
            const FRONT             = 0b01 << start_bit::FACE;
            const BACK              = 0b10 << start_bit::FACE;
        }
    }
}
mod image_flags {
    use super::*;
    use ::bevy::render::render_resource::{AddressMode, CompareFunction, FilterMode};

    #[rustfmt::skip]
    mod start_bit {
        pub(super) const V: u16      = 2;
        pub(super) const W: u16      = 2 + V;
        pub(super) const MAG: u16    = 2 + W;
        pub(super) const MIN: u16    = 1 + MAG;
        pub(super) const MIP: u16    = 1 + MIN;
        pub(super) const COMP: u16   = 1 + MIP;
        pub(super) const BORDER: u16 = 4 + COMP;
        pub(super) const _END: u16   = 3 + BORDER;
    }

    #[derive(Clone, Copy, Archive, Deserialize, Serialize)]
    pub struct ImageFlags(());
    impl ImageFlags {
        pub const fn mode_u(self) -> AddressMode {
            todo!()
        }
        pub const fn mode_v(self) -> AddressMode {
            todo!()
        }
        pub const fn mode_w(self) -> AddressMode {
            todo!()
        }
        pub const fn mag_filter(self) -> FilterMode {
            todo!()
        }
        pub const fn min_filter(self) -> FilterMode {
            todo!()
        }
        pub const fn mipmap_filter(self) -> FilterMode {
            todo!()
        }
        pub const fn compare(self) -> Option<CompareFunction> {
            todo!()
        }
    }

    bitflags::bitflags! {
        #[repr(transparent)]
        struct ImageFlagsInner: u16 {
            // address_mode_u: AddressMode
            const U_CLAMP_TO_EDGE          = 0b00;
            const U_REPEAT                 = 0b01;
            const U_MIRROR_REPEAT          = 0b10;
            const U_CLAMP_TO_BORDER        = 0b11;

            // address_mode_v: AddressMode
            const V_CLAMP_TO_EDGE          = 0b00 << start_bit::V;
            const V_REPEAT                 = 0b01 << start_bit::V;
            const V_MIRROR_REPEAT          = 0b10 << start_bit::V;
            const V_CLAMP_TO_BORDER        = 0b11 << start_bit::V;

            // address_mode_w: AddressMode
            const W_CLAMP_TO_EDGE          = 0b00 << start_bit::W;
            const W_REPEAT                 = 0b01 << start_bit::W;
            const W_MIRROR_REPEAT          = 0b10 << start_bit::W;
            const W_CLAMP_TO_BORDER        = 0b11 << start_bit::W;

            // mag_filter: FilterMode
            const MAG_NEAREST              = 0 << start_bit::MAG;
            const MAG_LINEAR               = 1 << start_bit::MAG;

            // min_filter: FilterMode
            const MIN_NEAREST              = 0 << start_bit::MIN;
            const MIN_LINEAR               = 1 << start_bit::MIN;

            // mipmap_filter: FilterMode
            const MIPMAP_NEAREST           = 0 << start_bit::MIP;
            const MIPMAP_LINEAR            = 1 << start_bit::MIP;

            // compare: Option<CompareFunction>
            const COMP_NONE                = 0b0000 << start_bit::COMP;
            const COMP_NEVER               = 0b0001 << start_bit::COMP;
            const COMP_LESS                = 0b0010 << start_bit::COMP;
            const COMP_EQUAL               = 0b0011 << start_bit::COMP;
            const COMP_LESS_EQUAL          = 0b0100 << start_bit::COMP;
            const COMP_GREATER             = 0b0101 << start_bit::COMP;
            const COMP_NOT_EQUAL           = 0b0110 << start_bit::COMP;
            const COMP_GREATER_EQUAL       = 0b0111 << start_bit::COMP;
            const COMP_ALWAYS              = 0b1000 << start_bit::COMP;

            // border_color: Option<SamplerBorderColor>
            const BORDER_NONE              = 0b000 << start_bit::BORDER;
            const BORDER_TRANSPARENT_BLACK = 0b001 << start_bit::BORDER;
            const BORDER_OPAQUE_BLACK      = 0b010 << start_bit::BORDER;
            const BORDER_OPAQUE_WHITE      = 0b011 << start_bit::BORDER;
            const BORDER_ZERO              = 0b100 << start_bit::BORDER;
        }
    }
}
pub use image_flags::ImageFlags;
pub use material_flags::MaterialFlags;

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub enum ParallaxMappingMethod {
    Occlusion,
    Relief { relief: NonZeroU16 },
}
#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Color([u8; 4]);
impl ArchivedColor {
    fn to_bevy(&self) -> bevy::Color {
        let [r, g, b, a] = self.0;
        bevy::Color::rgba_u8(r, g, b, a)
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct MeshId(u32);
impl ArchivedMeshId {
    /// SAFETY: `handles` must be larger than value contained therein
    pub unsafe fn pick<'a, T>(&self, handles: &'a [T]) -> &'a T {
        let index = usize::try_from(self.0).unwrap();
        // SAFETY: upheld by function invariant
        unsafe { handles.get_unchecked(index) }
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct MaterialId(u32);
impl ArchivedMaterialId {
    /// SAFETY: `handles` must be larger than value contained therein
    pub unsafe fn pick<'a, T>(&self, handles: &'a [T]) -> &'a T {
        let index = usize::try_from(self.0).unwrap();
        // SAFETY: upheld by function invariant
        unsafe { handles.get_unchecked(index) }
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct ImageId(NonZeroU32);
impl ArchivedImageId {
    /// SAFETY: `handles` must be larger than value contained therein
    unsafe fn pick<'a, T>(&self, handles: &'a [T]) -> &'a T {
        let index = usize::try_from(self.0.get() - 1).unwrap();

        // SAFETY: upheld by function invariant
        unsafe { handles.get_unchecked(index) }
    }
}
impl ImageId {
    pub fn new(index: u32) -> Self {
        ImageId(NonZeroU32::new(index.saturating_add(1)).unwrap())
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
impl ArchivedTransform {
    pub fn to_bevy(&self) -> bevy::Transform {
        bevy::Transform {
            translation: self.translation.into(),
            rotation: bevy::Quat::from_array(self.rotation),
            scale: self.scale.into(),
        }
    }
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Entity {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub children: u32,
    pub transform: Transform,
}

#[derive(Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Image {
    pub flags: ImageFlags,
}

#[derive(Archive, Deserialize, Serialize)]
pub struct Material {
    pub base_color: Color,
    pub emissive: Color,

    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub depth_bias: f32,
    pub parallax_depth_scale: f32,
    pub max_parallax_layer_count: f32,
    pub alpha_mode_mask: f32,

    pub flags: MaterialFlags,
    pub parallax_mapping_method: ParallaxMappingMethod,

    pub base_color_texture: Option<ImageId>,
    pub emissive_texture: Option<ImageId>,
    pub metallic_roughness_texture: Option<ImageId>,
    pub normal_map_texture: Option<ImageId>,
    pub occlusion_texture: Option<ImageId>,
    pub depth_map: Option<ImageId>,
}
impl ArchivedMaterial {
    const fn flip_normal_map_y(&self) -> bool {
        self.flags.flip_normal_map_y()
    }
    const fn double_sided(&self) -> bool {
        self.flags.double_sided()
    }
    const fn unlit(&self) -> bool {
        self.flags.unlit()
    }
    const fn fog_enabled(&self) -> bool {
        self.flags.fog_enabled()
    }
    const fn alpha_mode(&self) -> bevy::AlphaMode {
        self.flags.alpha_mode(self.alpha_mode_mask)
    }
    const fn cull_mode(&self) -> Option<Face> {
        self.flags.cull_mode()
    }
    const fn parallax_mapping_method(&self) -> bevy::ParallaxMappingMethod {
        use self::bevy::ParallaxMappingMethod::{Occlusion, Relief};
        use ArchivedParallaxMappingMethod as Arch;

        match self.parallax_mapping_method {
            Arch::Occlusion => Occlusion,
            Arch::Relief { relief } => Relief { max_steps: relief.get() as u32 },
        }
    }
    /// SAFETY: `images` must have the same size as the `images` field from the scene in which this
    /// `Material` exists.
    pub unsafe fn to_bevy(&self, images: &[bevy::Handle<bevy::Image>]) -> bevy::StandardMaterial {
        let get_image = |image: &Archived<Option<ImageId>>| {
            // SAFETY: From method's safety invariants,
            // `ImageId` is necessarilly an index of `images`
            image.as_ref().map(|i| unsafe { i.pick(images) }.clone())
        };
        bevy::StandardMaterial {
            base_color: self.base_color.to_bevy(),
            base_color_texture: get_image(&self.base_color_texture),
            emissive: self.emissive.to_bevy(),
            emissive_texture: get_image(&self.emissive_texture),
            perceptual_roughness: self.perceptual_roughness,
            metallic: self.metallic,
            metallic_roughness_texture: get_image(&self.metallic_roughness_texture),
            reflectance: self.reflectance,
            normal_map_texture: get_image(&self.normal_map_texture),
            flip_normal_map_y: self.flags.flip_normal_map_y(),
            occlusion_texture: get_image(&self.occlusion_texture),
            double_sided: self.flags.double_sided(),
            cull_mode: self.flags.cull_mode(),
            unlit: self.flags.unlit(),
            fog_enabled: self.flags.fog_enabled(),
            alpha_mode: self.flags.alpha_mode(self.alpha_mode_mask),
            depth_bias: self.depth_bias,
            depth_map: get_image(&self.depth_map),
            parallax_depth_scale: self.parallax_depth_scale,
            parallax_mapping_method: self.parallax_mapping_method(),
            max_parallax_layer_count: self.max_parallax_layer_count,
        }
    }
}

#[derive(Archive, Deserialize, Serialize)]
pub struct Scene {
    pub version: u16,
    pub materials: Box<[Material]>,
    pub entities: Box<[Entity]>,
}
