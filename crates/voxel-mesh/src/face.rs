/// The six directions a voxel face can point to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl Face {
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];

    /// Axis the face points along: 0 is x, 1 is y, 2 is z.
    #[must_use]
    pub const fn axis(self) -> usize {
        match self {
            Self::PosX | Self::NegX => 0,
            Self::PosY | Self::NegY => 1,
            Self::PosZ | Self::NegZ => 2,
        }
    }

    #[must_use]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::PosX | Self::PosY | Self::PosZ)
    }

    /// The in-plane axes (u, v) of the face. `u × v` points along the positive axis.
    #[must_use]
    pub const fn plane_axes(self) -> (usize, usize) {
        let axis = self.axis();
        ((axis + 1) % 3, (axis + 2) % 3)
    }

    /// Unit vector out of the voxel through this face.
    #[must_use]
    pub const fn normal(self) -> [i8; 3] {
        let sign = if self.is_positive() { 1 } else { -1 };
        let mut normal = [0; 3];
        normal[self.axis()] = sign;
        normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_axes_and_normal_form_a_right_handed_basis() {
        for face in Face::ALL {
            let (u, v) = face.plane_axes();
            assert_eq!((u + 1) % 3, v, "{face:?}");
            assert_ne!(u, face.axis());
            assert_eq!(face.normal().iter().map(|n| n.abs()).sum::<i8>(), 1);
        }
    }
}
