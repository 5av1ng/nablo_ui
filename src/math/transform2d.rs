//! Transform2D represents a 2D transformation matrix.

use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign};

use super::vec2::Vec2;

/// A 2D transformation matrix.
/// 
/// The formula of a 2D transformation matrix is:
/// $$
/// \begin{bmatrix}
/// m00 & m10 & m20 \\
/// m01 & m11 & m21 \\
///  0  &  0  &  1  \\
/// \end{bmatrix}
/// $$
/// 
/// The matrix multiplication implemented here is not the matrix multiplication in the mathematical sense, 
/// but simply multiplying each component individually.
/// To perform matrix multiplication in the mathematical sense, 
/// you can use `A >> B` to represent `AB`,
/// or `A << B` to represent `BA`
/// Similarly, to apply this matrix to a vector, you can use `A >> v` or `v << A` to represent `Av`.
/// 
/// In addition, the division implemented for this matrix also simply divides each component individually, 
/// rather than multiplying by the inverse of the matrix.
/// 
/// You can use indexing to access the components of the matrix, 
/// and the `Default` trait to create an identity matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Transform2D([[f32; 2]; 3]);

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Index<usize> for Transform2D {
    type Output = [f32; 2];

    fn index(&self, index: usize) -> &[f32; 2] {
        &self.0[index]
    }
}

impl IndexMut<usize> for Transform2D {
    fn index_mut(&mut self, index: usize) -> &mut [f32; 2] {
        &mut self.0[index]
    }
}

impl Transform2D {
	pub const ZERO: Self = Self::column_major(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
	pub const IDENTITY: Self = Self::column_major(
		1.0, 0.0, 0.0, 
		0.0, 1.0, 0.0
	);
	
	/// Creates a new 2D transformation matrix in column-major order.
	pub const fn column_major(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
		Self([[a, d], [b, e], [c, f]])
	}

	/// Creates a new 2D transformation matrix in row-major order.
	pub const fn row_major(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
		Self([[a, b], [c, d], [e, f]])
	}

	// /// Creates a new 2D transformation matrix with the given components.
	// /// 
	// /// The matrix is in the form:
	// /// $$
	// /// \begin{bmatrix}
	// /// a & b & tx \\
	// /// c & d & ty \\
	// /// 0 & 0 & 1 
	// /// \end{bmatrix}
	// pub const fn new(a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32) -> Self {
	// 	Self([[a, c], [b, d], [tx, ty]])
	// }

	/// Creates a new 2D transformation matrix that scales by the given factors.
	pub fn scale(factor: impl Into<Vec2>) -> Self {
		let factor = factor.into();
		Self::column_major(
			factor.x, 0.0, 0.0, 
			0.0, factor.y, 0.0
		)
	}

	/// Creates a new 2D transformation matrix that rotates by the given angle in radians.
	pub fn rotate(angle: f32) -> Self {
		let cos = angle.cos();
		let sin = angle.sin();
		Self::column_major(
			cos, -sin, 0.0, 
			sin, cos, 0.0
		)
	}

	/// Creates a new 2D transformation matrix that rotates by the given angle in degrees.
	pub fn rotate_degrees(angle: f32) -> Self {
		Self::rotate(angle.to_radians())
	}

	/// Creates a new 2D transformation matrix that translates by the given vector.
	pub fn translate(translation: impl Into<Vec2>) -> Self {
		let translation = translation.into();
		Self::column_major(
			1.0, 0.0, translation.x, 
			0.0, 1.0, translation.y
		)
	}

	/// Calculates the inverse of the transformation matrix.
	pub fn inverse(self) -> Self {
		let c11 = self[1][1];
		let c12 = - self[0][1];
		let c21 = - self[1][0];
		let c22 = self[0][0];
		let c31 = self[1][0] * self[2][1] - self[2][0] * self[1][1];
		let c32 = self[0][0] * self[2][1] - self[2][0] * self[0][1];
		let det = self[0][0] * self[1][1] - self[0][1] * self[1][0];
		Self::column_major(
			c11 / det, c21 / det, c31 / det, 
			c12 / det, c22 / det, c32 / det
		)
	}
}

impl Add for Transform2D {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = self[i][j] + other[i][j];
			}
		}
		result
	}
}

impl Sub for Transform2D {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = self[i][j] - other[i][j];
			}
		}
		result
	}
}

impl Mul for Transform2D {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = other[i][j] * self[i][j]
			}
		}
		result
	}
}

impl Mul<f32> for Transform2D {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = self[i][j] * other;
			}
		}
		result
	}
}

impl Mul<Transform2D> for f32 {
	type Output = Transform2D;

	fn mul(self, other: Transform2D) -> Transform2D {
		other * self
	}
}

impl Div<f32> for Transform2D {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = self[i][j] / other;
			}
		}
		result
	}
}

impl Div for Transform2D {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		let mut result = Self::ZERO;
		for i in 0..3 {
			for j in 0..2 {
				result[i][j] = self[i][j] / other[i][j];
			}
		}
		result
	}
}

impl AddAssign for Transform2D {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl SubAssign for Transform2D {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

impl MulAssign for Transform2D {
	fn mul_assign(&mut self, other: Self) {
		*self = *self * other;
	}
}

impl MulAssign<f32> for Transform2D {
	fn mul_assign(&mut self, other: f32) {
		*self = *self * other;
	}
}

impl DivAssign<f32> for Transform2D {
	fn div_assign(&mut self, other: f32) {
		*self = *self / other;
	}
}

impl DivAssign for Transform2D {
	fn div_assign(&mut self, other: Self) {
		*self = *self / other;
	}
}

impl Shr for Transform2D {
	type Output = Self;

	fn shr(self, other: Self) -> Self {
		Self::column_major(
			self[0][0] * other[0][0] + self[1][0] * other[0][1],
			self[0][0] * other[1][0] + self[1][0] * other[1][1],
			self[0][0] * other[2][0] + self[1][0] * other[2][1] + self[2][0],
			self[0][1] * other[0][0] + self[1][1] * other[0][1],
			self[0][1] * other[1][0] + self[1][1] * other[1][1],
			self[0][1] * other[2][0] + self[1][1] * other[2][1] + self[2][1],
		)
	}
}

impl Shl<Transform2D> for Vec2 { 
	type Output = Vec2;

	#[allow(clippy::suspicious_arithmetic_impl)]
	fn shl(self, other: Transform2D) -> Vec2 {
		other >> self
	}
}

impl ShlAssign for Transform2D {
	fn shl_assign(&mut self, other: Self) {
		*self = *self << other;
	}
}

impl ShlAssign<Transform2D> for Vec2 {
	fn shl_assign(&mut self, other: Transform2D) {
		*self = *self << other;
	}
}

impl Shl for Transform2D {
	type Output = Self;

	#[allow(clippy::suspicious_arithmetic_impl)]
	fn shl(self, other: Self) -> Self {
		other >> self
	}
}

impl Shr<Vec2> for Transform2D {
	type Output = Vec2;

	fn shr(self, other: Vec2) -> Vec2 {
		let mut result = Vec2::ZERO;
		result.x = self[0][0] * other.x + self[1][0] * other.y + self[2][0];
		result.y = self[0][1] * other.x + self[1][1] * other.y + self[2][1];
		result
	}
}

impl ShrAssign for Transform2D {
	fn shr_assign(&mut self, other: Self) {
		*self = *self >> other;
	}
}


impl From<[[f32; 2]; 3]> for Transform2D {
	fn from(array: [[f32; 2]; 3]) -> Self {
		Self(array)
	}
}

impl From<[Vec2; 3]> for Transform2D {
	fn from(array: [Vec2; 3]) -> Self {
		Self::row_major(array[0].x, array[0].y, array[1].x, array[1].y, array[2].x, array[2].y)
	}
}

impl From<[f32; 6]> for Transform2D {
	fn from(array: [f32; 6]) -> Self {
		Self::column_major(array[0], array[1], array[2], array[3], array[4], array[5])
	}
}

/// Creates a new 2D transformation matrix.
pub fn transform2d(m00: f32, m01: f32, m02: f32, m10: f32, m11: f32, m12: f32) -> Transform2D {
	Transform2D::column_major(m00, m01, m02, m10, m11, m12)
}

mod test {
	#[test]
	fn test_mul() {
		use crate::prelude::Vec2;
		use crate::prelude::Transform2D;

		let lhs = Transform2D::column_major(
			1.0, 2.0, 3.0, 
			4.0, 5.0, 6.0
		);
		let rhs = Transform2D::column_major(
			7.0, 8.0, 9.0, 
			10.0, 11.0, 12.0
		);
		let vec = Vec2::new(7.0, 8.0);
		let expected_l = Transform2D::column_major(
			27.0, 30.0, 36.0, 
			78.0, 87.0, 102.0
		);
		let expected_r = Transform2D::column_major(
			39.0, 54.0, 78.0,
			54.0, 75.0, 108.0
		);
		assert_eq!(lhs >> rhs, expected_l);
		assert_eq!(lhs << rhs, expected_r);
		assert_eq!(lhs >> vec, Vec2::new(26.0, 74.0));
	}
}