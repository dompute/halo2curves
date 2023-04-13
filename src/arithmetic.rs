//! This module provides common utilities, traits and structures for group and
//! field arithmetic.
//!
//! This module is temporary, and the extension traits defined here are expected to be
//! upstreamed into the `ff` and `group` crates after some refactoring.

pub trait CurveAffineExt: pasta_curves::arithmetic::CurveAffine {
    fn batch_add<const COMPLETE: bool, const LOAD_POINTS: bool>(
        points: &mut [Self],
        output_indices: &[u32],
        num_points: usize,
        offset: usize,
        bases: &[Self],
        base_positions: &[u32],
    );

    /// Unlike the `Coordinates` trait, this just returns the raw affine coordinates without checking `is_on_curve`
    fn into_coordinates(self) -> (Self::Base, Self::Base) {
        // fallback implementation
        let coordinates = self.coordinates().unwrap();
        (*coordinates.x(), *coordinates.y())
    }
}

/// Compute a + b + carry, returning the result and the new carry over.
#[inline(always)]
pub(crate) const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + (b as u128) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}

/// Compute a - (b + borrow), returning the result and the new borrow.
#[inline(always)]
pub(crate) const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let ret = (a as u128).wrapping_sub((b as u128) + ((borrow >> 63) as u128));
    (ret as u64, (ret >> 64) as u64)
}

/// Compute a + (b * c) + carry, returning the result and the new carry over.
#[inline(always)]
pub(crate) const fn mac(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
    let ret = (a as u128) + ((b as u128) * (c as u128)) + (carry as u128);
    (ret as u64, (ret >> 64) as u64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn u64_and_test() {
        assert_eq!(0 & 0, 0);
        assert_eq!(0 & 1, 0);
        assert_eq!(1 & 0, 0);
        assert_eq!(1 & 1, 1);
        assert_eq!(u64::MAX & 0, 0);
        assert_eq!(u64::MAX & 1, 1);
        assert_eq!(u64::MAX & u64::MAX, u64::MAX);
    }

    #[test]
    fn test_abc() {
        assert_eq!(super::adc(0, 0, 0), (0, 0));
        assert_eq!(super::adc(0, 0, 1), (1, 0));
        assert_eq!(super::adc(0, 1, 0), (1, 0));
        assert_eq!(super::adc(0, 1, 1), (2, 0));
        assert_eq!(super::adc(1, 0, 0), (1, 0));
        assert_eq!(super::adc(1, 0, 1), (2, 0));
        assert_eq!(super::adc(1, 1, 0), (2, 0));
        assert_eq!(super::adc(1, 1, 1), (3, 0));
        assert_eq!(super::adc(u64::MAX, 0, 0), (u64::MAX, 0));
        assert_eq!(super::adc(u64::MAX, 0, 1), (0, 1));
        assert_eq!(super::adc(u64::MAX, 1, 0), (0, 1));
        assert_eq!(super::adc(u64::MAX, 1, 1), (1, 1));
        assert_eq!(super::adc(u64::MAX, u64::MAX, 0), (u64::MAX - 1, 1));
        assert_eq!(super::adc(u64::MAX, u64::MAX, 1), (u64::MAX, 1));
        assert_eq!(super::adc(u64::MAX, u64::MAX, 2), (0, 2));
    }

    #[test]
    fn test_mac() {
        assert_eq!(super::mac(0, 0, 0, 0), (0, 0));
        assert_eq!(super::mac(0, 0, 0, 1), (1, 0));
        assert_eq!(super::mac(0, 0, 1, 0), (0, 0));
        assert_eq!(super::mac(0, 0, 1, 1), (1, 0));
        assert_eq!(super::mac(0, 1, 0, 0), (0, 0));
        assert_eq!(super::mac(u64::MAX, 0, 0, 0), (u64::MAX, 0));
        assert_eq!(super::mac(u64::MAX, u64::MAX, u64::MAX, 0), (0, u64::MAX));
        assert_eq!(
            super::mac(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
            (u64::MAX, u64::MAX)
        );
    }

    #[test]
    fn test_wrapping_sub() {
        assert_eq!(0u128.wrapping_sub(1), u128::MAX);
        assert_eq!(0u128.wrapping_sub(2), u128::MAX - 1);
        assert_eq!(1u128.wrapping_sub(1), 0);
        assert_eq!(2u128.wrapping_sub(1), 1);

        println!("u128::MAX = {}", u128::MAX);

        assert_eq!(u128::MAX as u64, u64::MAX);
        assert_eq!(((u64::MAX as u128) + 1) as u64, 0);
        assert_eq!(((u64::MAX as u128) + 2) as u64, 1);
        assert_eq!(((u64::MAX as u128) + 3) as u64, 2);
    }

    #[test]
    fn test_wrapping_mul() {
        assert_eq!(0u128.wrapping_mul(0), 0);
        assert_eq!(0u128.wrapping_mul(1), 0);
        assert_eq!(u64::MAX.wrapping_mul(1), u64::MAX);
        assert_eq!(u64::MAX.wrapping_mul(2), u64::MAX - 1);
        assert_eq!(u64::MAX.wrapping_mul(3), u64::MAX - 2);
        assert_eq!((u64::MAX - 1).wrapping_mul(2), u64::MAX - 3);
        assert_eq!((u64::MAX - 2).wrapping_mul(2), u64::MAX - 5);
    }

    #[test]
    fn test_sbb() {
        assert_eq!(super::sbb(0, 0, 0), (0, 0));
        assert_eq!(super::sbb(0, 0, 1), (0, 0));
        assert_eq!(super::sbb(0, 1, 0), (u64::MAX, u64::MAX));
        assert_eq!(super::sbb(0, 1, 1), (u64::MAX, u64::MAX));
        assert_eq!(super::sbb(1, 0, 0), (1, 0));
        assert_eq!(super::sbb(1, 0, 1), (1, 0));
        assert_eq!(super::sbb(1, 1, 0), (0, 0));
        assert_eq!(super::sbb(1, 1, 1), (0, 0));
    }
}
