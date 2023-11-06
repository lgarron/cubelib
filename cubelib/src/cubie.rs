use std::fmt::{Display, Formatter};

use crate::cube::Color::*;
use crate::cube::CornerPosition::*;
use crate::cube::EdgePosition::*;
use crate::cube::Face::*;
use crate::cube::{
    Color, Corner, CornerPosition, Cube, Edge, EdgePosition, Invertible, Move, NewSolved,
    Transformation, Turnable,
};

//http://kociemba.org/math/cubielevel.htm
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct CubieCube {
    pub edges: EdgeCubieCube,
    pub corners: CornerCubieCube,
}

impl CubieCube {
    pub fn new(edges: EdgeCubieCube, corners: CornerCubieCube) -> CubieCube {
        CubieCube { edges, corners }
    }

    #[cfg(target_arch = "avx2")]
    pub fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

impl Display for CubieCube {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_display(f)
    }
}

impl Turnable for CubieCube {
    #[inline]
    fn turn(&mut self, m: Move) {
        self.edges.turn(m);
        self.corners.turn(m);
    }

    #[inline]
    fn transform(&mut self, t: Transformation) {
        let Transformation(_axis, _turn) = t;
        self.edges.transform(t);
        self.corners.transform(t);
    }
}

impl Invertible for CubieCube {
    #[inline]
    fn invert(&mut self) {
        self.edges.invert();
        self.corners.invert();
    }
}

impl NewSolved for CubieCube {
    #[inline]
    fn new_solved() -> CubieCube {
        CubieCube {
            edges: EdgeCubieCube::new_solved(),
            corners: CornerCubieCube::new_solved(),
        }
    }
}

//One byte per edge, 4 bits for id, 3 bits for eo (UD/FB/RL), 1 bit free
//UB UR UF UL FR FL BR BL DF DR DB DL
#[derive(Debug, Clone, Copy)]
pub struct EdgeCubieCube(
    #[cfg(target_feature = "avx2")] pub core::arch::x86_64::__m128i,
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    pub core::arch::wasm32::v128,
);

impl EdgeCubieCube {
    pub(crate) const BAD_EDGE_MASK_UD: u64 = 0x0808080808080808;
    pub(crate) const BAD_EDGE_MASK_FB: u64 = 0x0404040404040404;
    pub(crate) const BAD_EDGE_MASK_RL: u64 = 0x0202020202020202;

    #[cfg(target_feature = "avx2")]
    pub fn new(state: std::arch::x86_64::__m128i) -> EdgeCubieCube {
        EdgeCubieCube(state)
    }

    #[cfg(target_feature = "avx2")]
    pub fn get_edges(&self) -> [Edge; 12] {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_get_edges(self) }
    }

    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    pub fn get_edges(&self) -> [Edge; 12] {
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::get_edges(self)
    }

    #[cfg(target_feature = "avx2")]
    pub fn get_edges_raw(&self) -> [u64; 2] {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_get_edges_raw(self) }
    }

    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    pub fn get_edges_raw(&self) -> [u64; 2] {
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::get_edges_raw(self)
    }
}

#[cfg(feature = "serde_support")]
impl serde::Serialize for EdgeCubieCube {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let bytes = [0_u8; 16];
        unsafe {
            #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
            std::arch::wasm32::v128_store(bytes.as_ptr() as *mut std::arch::wasm32::v128, self.0);
            #[cfg(target_feature = "avx2")]
            std::arch::x86_64::_mm_store_si128(bytes.as_ptr() as *mut std::arch::x86_64::__m128i, self.0);
        }
        serializer.serialize_bytes(&bytes)
    }
}

#[cfg(feature = "serde_support")]
struct EdgeCubieCubeVisitor;

#[cfg(feature = "serde_support")]
impl<'de> serde::de::Visitor<'de> for EdgeCubieCubeVisitor {
    type Value = EdgeCubieCube;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array of length 16")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: serde::de::Error {
        if v.len() != 16 {
            Err(E::custom("Array length must be 16"))
        } else {
            let val = unsafe {
                #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
                let val = std::arch::wasm32::v128_load(v.as_ptr() as *const std::arch::wasm32::v128);
                #[cfg(target_feature = "avx2")]
                let val = std::arch::x86_64::_mm_load_si128(v.as_ptr() as *const std::arch::x86_64::__m128i);
                val
            };
            Ok(EdgeCubieCube(val))
        }
    }
}

#[cfg(feature = "serde_support")]
impl<'de> serde::Deserialize<'de> for EdgeCubieCube {
    fn deserialize<D>(deserializer: D) -> Result<EdgeCubieCube, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(EdgeCubieCubeVisitor)
    }
}

impl PartialEq for EdgeCubieCube {

    fn eq(&self, other: &Self) -> bool {
        let a = self.get_edges_raw();
        let b = other.get_edges_raw();
        a.eq(&b)
    }
}

impl Turnable for EdgeCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_turn(self, face, turn);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::turn(self, face, turn)
    }

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn transform(&mut self, t: Transformation) {
        let Transformation(axis, turn) = t;
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_transform(self, axis, turn);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn transform(&mut self, t: Transformation) {
        let Transformation(axis, turn) = t;
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::transform(self, axis, turn)
    }
}

impl Invertible for EdgeCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn invert(&mut self) {
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_invert(self);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn invert(&mut self) {
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::invert(self)
    }
}

impl NewSolved for EdgeCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn new_solved() -> Self {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2EdgeCubieCube::unsafe_new_solved() }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn new_solved() -> Self {
        crate::wasm32_cubie::wasm32_cubie::WASM32EdgeCubieCube::new_solved()
    }
}

//One byte per corner, 3 bits for id, 2 bits free, 3 bits for co (from UD perspective)
//UBL UBR UFR UFL DFL DFR DBR DBL
#[derive(Debug, Clone, Copy)]
pub struct CornerCubieCube(
    #[cfg(target_feature = "avx2")] pub core::arch::x86_64::__m128i,
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    pub core::arch::wasm32::v128,
);

impl CornerCubieCube {
    #[cfg(target_feature = "avx2")]
    pub fn new(state: std::arch::x86_64::__m128i) -> CornerCubieCube {
        CornerCubieCube(state)
    }

    #[inline]
    #[cfg(target_feature = "avx2")]
    pub fn get_corners(&self) -> [Corner; 8] {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_get_corners(self) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn get_corners(&self) -> [Corner; 8] {
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::get_corners(self)
    }

    #[inline]
    #[cfg(target_feature = "avx2")]
    pub fn get_corners_raw(&self) -> u64 {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_get_corners_raw(self) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    pub fn get_corners_raw(&self) -> u64 {
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::get_corners_raw(self)
    }
}

#[cfg(feature = "serde_support")]
impl serde::Serialize for CornerCubieCube {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let bytes = [0_u8; 16];
        unsafe {
            #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
            std::arch::wasm32::v128_store(bytes.as_ptr() as *mut std::arch::wasm32::v128, self.0);
            #[cfg(target_feature = "avx2")]
            std::arch::x86_64::_mm_store_si128(bytes.as_ptr() as *mut std::arch::x86_64::__m128i, self.0);
        }
        serializer.serialize_bytes(&bytes)
    }
}

#[cfg(feature = "serde_support")]
struct CornerCubieCubeVisitor;

#[cfg(feature = "serde_support")]
impl<'de> serde::de::Visitor<'de> for CornerCubieCubeVisitor {
    type Value = CornerCubieCube;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array of length 16")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: serde::de::Error {
        if v.len() != 16 {
            Err(E::custom("Array length must be 16"))
        } else {
            let val = unsafe {
                #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
                    let val = std::arch::wasm32::v128_load(v.as_ptr() as *const std::arch::wasm32::v128);
                #[cfg(target_feature = "avx2")]
                    let val = std::arch::x86_64::_mm_load_si128(v.as_ptr() as *const std::arch::x86_64::__m128i);
                val
            };
            Ok(CornerCubieCube(val))
        }
    }
}

#[cfg(feature = "serde_support")]
impl<'de> serde::Deserialize<'de> for CornerCubieCube {
    fn deserialize<D>(deserializer: D) -> Result<CornerCubieCube, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CornerCubieCubeVisitor)
    }
}

impl PartialEq for CornerCubieCube {

    fn eq(&self, other: &Self) -> bool {
        let a = self.get_corners_raw();
        let b = other.get_corners_raw();
        a.eq(&b)
    }
}

impl Turnable for CornerCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_turn(self, face, turn);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::turn(self, face, turn);
    }

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn transform(&mut self, t: Transformation) {
        let Transformation(axis, turn) = t;
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_transform(self, axis, turn);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn transform(&mut self, t: Transformation) {
        let Transformation(axis, turn) = t;
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::transform(self, axis, turn);
    }
}

impl Invertible for CornerCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn invert(&mut self) {
        unsafe {
            crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_invert(self);
        }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn invert(&mut self) {
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::invert(self);
    }
}

impl NewSolved for CornerCubieCube {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn new_solved() -> Self {
        unsafe { crate::avx2_cubie::avx2_cubie::AVX2CornerCubieCube::unsafe_new_solved() }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn new_solved() -> Self {
        crate::wasm32_cubie::wasm32_cubie::WASM32CornerCubieCube::new_solved()
    }
}

impl Cube for CubieCube {
    fn get_facelets(&self) -> [[Color; 9]; 6] {
        let corners = self.corners.get_corners();
        let edges = self.edges.get_edges();
        let mut facelets = [[None; 9]; 6];

        //There has to be a better way
        let c = |id: CornerPosition, twist: u8| {
            let corner = corners[id as usize];
            let twist_id = (3 - corner.orientation + twist) % 3;
            CubieCube::CORNER_COLORS[corner.id as usize][twist_id as usize]
        };

        let e = |id: EdgePosition, flip: bool| {
            let edge = edges[id as usize];
            let eo_id = !(edge.oriented_fb ^ flip) as usize;
            CubieCube::EDGE_COLORS[edge.id as usize][eo_id as usize]
        };

        facelets[Up][0] = c(UBL, 0);
        facelets[Up][1] = e(UB, false);
        facelets[Up][2] = c(UBR, 0);
        facelets[Up][3] = e(UL, false);
        facelets[Up][4] = White;
        facelets[Up][5] = e(UR, false);
        facelets[Up][6] = c(UFL, 0);
        facelets[Up][7] = e(UF, false);
        facelets[Up][8] = c(UFR, 0);

        facelets[Down][0] = c(DFL, 0);
        facelets[Down][1] = e(DF, false);
        facelets[Down][2] = c(DFR, 0);
        facelets[Down][3] = e(DL, false);
        facelets[Down][4] = Yellow;
        facelets[Down][5] = e(DR, false);
        facelets[Down][6] = c(DBL, 0);
        facelets[Down][7] = e(DB, false);
        facelets[Down][8] = c(DBR, 0);

        facelets[Front][0] = c(UFL, 1);
        facelets[Front][1] = e(UF, true);
        facelets[Front][2] = c(UFR, 2);
        facelets[Front][3] = e(FL, false);
        facelets[Front][4] = Green;
        facelets[Front][5] = e(FR, false);
        facelets[Front][6] = c(DFL, 2);
        facelets[Front][7] = e(DF, true);
        facelets[Front][8] = c(DFR, 1);

        facelets[Back][0] = c(UBR, 1);
        facelets[Back][1] = e(UB, true);
        facelets[Back][2] = c(UBL, 2);
        facelets[Back][3] = e(BR, false);
        facelets[Back][4] = Blue;
        facelets[Back][5] = e(BL, false);
        facelets[Back][6] = c(DBR, 2);
        facelets[Back][7] = e(DB, true);
        facelets[Back][8] = c(DBL, 1);

        facelets[Left][0] = c(UBL, 1);
        facelets[Left][1] = e(UL, true);
        facelets[Left][2] = c(UFL, 2);
        facelets[Left][3] = e(BL, true);
        facelets[Left][4] = Orange;
        facelets[Left][5] = e(FL, true);
        facelets[Left][6] = c(DBL, 2);
        facelets[Left][7] = e(DL, true);
        facelets[Left][8] = c(DFL, 1);

        facelets[Right][0] = c(UFR, 1);
        facelets[Right][1] = e(UR, true);
        facelets[Right][2] = c(UBR, 2);
        facelets[Right][3] = e(FR, true);
        facelets[Right][4] = Red;
        facelets[Right][5] = e(BR, true);
        facelets[Right][6] = c(DFR, 2);
        facelets[Right][7] = e(DR, true);
        facelets[Right][8] = c(DBR, 1);

        facelets
    }
}

impl CubieCube {
    pub const CORNER_COLORS: [[Color; 3]; 8] = [
        [White, Orange, Blue],
        [White, Blue, Red],
        [White, Red, Green],
        [White, Green, Orange],
        [Yellow, Orange, Green],
        [Yellow, Green, Red],
        [Yellow, Red, Blue],
        [Yellow, Blue, Orange],
    ];

    const EDGE_COLORS: [[Color; 2]; 12] = [
        [White, Blue],
        [White, Red],
        [White, Green],
        [White, Orange],
        [Green, Red],
        [Green, Orange],
        [Blue, Red],
        [Blue, Orange],
        [Yellow, Green],
        [Yellow, Red],
        [Yellow, Blue],
        [Yellow, Orange],
    ];
}
