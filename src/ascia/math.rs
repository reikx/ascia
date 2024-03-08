use std::f32::consts::PI;

#[derive(Debug,Clone,Copy)]
#[repr(C)]
pub struct Vec2{
    pub x:f32,
    pub y:f32,
}

impl std::ops::Add<Vec2> for Vec2{
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        return Vec2{
            x:self.x + rhs.x,
            y:self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Vec2> for Vec2{
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        return Vec2{
            x:self.x - rhs.x,
            y:self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<Vec2> for Vec2{
    type Output = f32;
    fn mul(self, rhs: Vec2) -> Self::Output {
        return self.x * rhs.x + self.y * rhs.y;
    }
}

impl std::ops::Mul<f32> for Vec2{
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Self::Output {
        return Vec2{
            x:self.x * rhs,
            y:self.y * rhs,
        };
    }
}

impl std::ops::Mul<Vec2> for f32{
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        return Vec2{
            x:rhs.x * self,
            y:rhs.y * self,
        };
    }
}

impl std::default::Default for Vec2{
    fn default() -> Self {
        return Vec2{
            x:0.0,
            y:0.0,
        }
    }
}

impl Vec2{
    pub fn norm(&self) -> f32{
        return f32::sqrt(self.x * self.x + self.y * self.y);
    }

    pub fn normalize(&self) -> Self{
        let l = self.norm();
        return Vec2{
            x:self.x / l,
            y:self.y / l
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec3{
    pub x:f32,
    pub y:f32,
    pub z:f32
}
impl std::ops::Add<Vec3> for Vec3{
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        return Vec3{
            x:self.x + rhs.x,
            y:self.y + rhs.y,
            z:self.z + rhs.z
        }
    }
}

impl std::ops::Sub<Vec3> for Vec3{
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Self::Output {
        return Vec3{
            x:self.x - rhs.x,
            y:self.y - rhs.y,
            z:self.z - rhs.z
        }
    }
}

impl std::ops::Mul<Vec3> for Vec3{
    type Output = f32;
    fn mul(self, rhs: Vec3) -> Self::Output {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
}

impl std::ops::Mul<f32> for Vec3{
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self::Output {
        return Vec3{
            x:self.x * rhs,
            y:self.y * rhs,
            z:self.z * rhs
        };
    }
}

impl std::ops::Mul<Vec3> for f32{
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        return Vec3{
            x:rhs.x * self,
            y:rhs.y * self,
            z:rhs.z * self
        };
    }
}

impl std::ops::Div<f32> for Vec3{
    type Output = Vec3;
    fn div(self, rhs: f32) -> Self::Output {
        return Vec3{
            x:self.x / rhs,
            y:self.y / rhs,
            z:self.z / rhs,
        };
    }
}

impl std::ops::BitXor<Vec3> for Vec3{
    type Output = Vec3;
    fn bitxor(self, rhs: Vec3) -> Self::Output {
        return Vec3{
            x:self.y * rhs.z - self.z * rhs.y,
            y:self.z * rhs.x - self.x * rhs.z,
            z:self.x * rhs.y - self.y * rhs.x
        }
    }
}

impl std::default::Default for Vec3{
    fn default() -> Self {
        return Vec3{
            x:0.0,
            y:0.0,
            z:0.0
        }
    }
}

impl Vec3{
    pub fn norm(&self) -> f32{
        return f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
    }

    pub fn normalize(&self) -> Self{
        let l = self.norm();
        return Vec3{
            x:self.x / l,
            y:self.y / l,
            z:self.z / l
        }
    }

    #[inline]
    pub fn rotate_by(&self,rotator:&Quaternion) -> Self{
        return rotator.rotate(self);
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
#[repr(C)]
pub struct Vec4{
    pub w:f32,
    pub x:f32,
    pub y:f32,
    pub z:f32
}

impl Vec4{
    pub fn norm(&self) -> f32{
        return f32::sqrt(self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z);
    }

    pub fn normalize(&self) -> Self{
        let l = self.norm();
        return Vec4{
            w:self.w / l,
            x:self.x / l,
            y:self.y / l,
            z:self.z / l
        }
    }
}

impl std::ops::Add<Vec4> for Vec4{
    type Output = Vec4;

    fn add(self, rhs: Vec4) -> Self::Output {
        return Vec4{
            w:self.w + rhs.w,
            x:self.x + rhs.x,
            y:self.y + rhs.y,
            z:self.z + rhs.z
        }
    }
}

impl std::ops::Sub<Vec4> for Vec4{
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Self::Output {
        return Vec4{
            w:self.w - rhs.w,
            x:self.x - rhs.x,
            y:self.y - rhs.y,
            z:self.z - rhs.z
        }
    }
}


#[derive(Debug,Clone,Copy,PartialEq)]
#[repr(C)]
pub struct Quaternion {
    pub vec4:Vec4
}

impl Quaternion{
    pub fn new(axis: &Vec3, rad:f32) -> Self{
        let s = f32::sin(rad / 2.0);
        let c = f32::cos(rad / 2.0);
        let a = axis.normalize();
        return Quaternion{
            vec4:Vec4{
                w:c,
                x:s * a.x,
                y:s * a.y,
                z:s * a.z,
            }
        }
    }

    pub fn rotate(&self,v:&Vec3) -> Vec3{
        let res = (*self * Quaternion{
            vec4:Vec4{
                w:0.0,
                x:v.x,
                y:v.y,
                z:v.z
            }
        }) * self.conjugate();
        return Vec3{
            x:res.vec4.x,
            y:res.vec4.y,
            z:res.vec4.z
        };
    }

    pub fn rotator(from:&Vec3, to:&Vec3) -> Self{
        let f = from.normalize();
        let t = to.normalize();
        let a = f ^ t;
        let b = f * t;
        let c = f * f;
        if f32::abs(a.x) <= f32::EPSILON && f32::abs(a.y) <= f32::EPSILON && f32::abs(a.z) <= f32::EPSILON{
            if b >= 0.0{
                return Quaternion{
                    vec4: Vec4 {
                        w: 1.0,
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                }
            }
            
            // gram schmidt orthonormalization
            
            let e1 = Vec3{
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
            let v1 = e1 - (e1 * f) * f;
            if f32::abs(v1.x) <= f32::EPSILON && f32::abs(v1.y) <= f32::EPSILON && f32::abs(v1.z) <= f32::EPSILON{
                let e2 = Vec3{
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                };
                let v2 = e2 - (e2 * f) * f;
                return Quaternion::new(&v2, PI);
            }
            return Quaternion::new(&v1, PI);
        } 
        let d = (c + b) * (c + b);
        let e = a * a;
        let p = f32::sqrt(d / (d + e));
        let q = f32::sqrt(1.0 / (d + e));
        
        return Quaternion{
            vec4: Vec4 {
                w: p,
                x: q * a.x,
                y: q * a.y,
                z: q * a.z,
            },
        };
    }

    pub fn conjugate(&self) -> Self{
        return Quaternion{
            vec4:Vec4{
                w:self.vec4.w,
                x:-self.vec4.x,
                y:-self.vec4.y,
                z:-self.vec4.z
            }
        }
    }

    pub fn normalize(&self) -> Self{
        return Quaternion{vec4:self.vec4.normalize()};
    }
}

impl std::default::Default for Quaternion{
    fn default() -> Self {
        return Quaternion{
            vec4:Vec4{
                w:1.0,
                x:0.0,
                y:0.0,
                z:0.0
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use std::f32::consts::PI;
    use crate::ascia::math::{Quaternion, Vec3};
    
    #[test]
    pub fn test_rotate(){
        {
            let a = Vec3{
                x: 1.0,
                y: 3.0,
                z: 2.0,
            };
            let b = Vec3{
                x: 2.0,
                y: 3.0,
                z: -1.0,
            };
            let diff = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }, PI / 2.0).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON * 2.0);
            assert!(f32::abs(diff.y) <= f32::EPSILON * 2.0);
            assert!(f32::abs(diff.z) <= f32::EPSILON * 2.0);
        }
    }
    #[test]
    pub fn test_rotator(){
        {
            let a = Vec3{
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
            let b = Vec3{
                x: 0.0,
                y: -1.0,
                z: 0.0,
            };
            let diff = Quaternion::rotator(&a, &b).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON);
            assert!(f32::abs(diff.y) <= f32::EPSILON);
            assert!(f32::abs(diff.z) <= f32::EPSILON);
        }
        {
            let a = Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            };
            let b = Vec3{
                x: 0.0,
                y: 0.0,
                z: -1.0,
            };
            let diff = Quaternion::rotator(&a, &b).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON);
            assert!(f32::abs(diff.y) <= f32::EPSILON);
            assert!(f32::abs(diff.z) <= f32::EPSILON);
        }
        {
            let a = Vec3{
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
            let b = Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            };
            let diff = Quaternion::rotator(&a, &b).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON);
            assert!(f32::abs(diff.y) <= f32::EPSILON);
            assert!(f32::abs(diff.z) <= f32::EPSILON);
        }
        {
            let a = Vec3{
                x: 1.0,
                y: 1.0,
                z: 0.0,
            };
            let b = Vec3{
                x: 1.0,
                y: 0.0,
                z: 1.0,
            };
            let diff = Quaternion::rotator(&a, &b).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON);
            assert!(f32::abs(diff.y) <= f32::EPSILON);
            assert!(f32::abs(diff.z) <= f32::EPSILON);
        }
        {
            let a = Vec3{
                x: 1.0,
                y: 1.0,
                z: -2.0,
            };
            let b = Vec3{
                x: 1.0,
                y: -2.0,
                z: 1.0,
            };
            let diff = Quaternion::rotator(&a, &b).rotate(&a) - b;
            assert!(f32::abs(diff.x) <= f32::EPSILON);
            assert!(f32::abs(diff.y) <= f32::EPSILON);
            assert!(f32::abs(diff.z) <= f32::EPSILON);
        }
    }
}

impl std::ops::Add<Quaternion> for Quaternion{
    type Output = Quaternion;

    fn add(self, rhs: Quaternion) -> Self::Output {
        return Quaternion{
            vec4:self.vec4 + rhs.vec4
        };
    }
}

impl std::ops::Sub<Quaternion> for Quaternion{
    type Output = Quaternion;

    fn sub(self, rhs: Quaternion) -> Self::Output{
        return Quaternion{
            vec4:self.vec4 - rhs.vec4
        };
    }
}

impl std::ops::Mul<Quaternion> for Quaternion{
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output{
        return Quaternion{
            vec4:Vec4{
                w:
                self.vec4.w * rhs.vec4.w
                    - self.vec4.x * rhs.vec4.x
                    - self.vec4.y * rhs.vec4.y
                    - self.vec4.z * rhs.vec4.z,
                x:
                self.vec4.w * rhs.vec4.x
                    + self.vec4.x * rhs.vec4.w
                    + self.vec4.y * rhs.vec4.z
                    - self.vec4.z * rhs.vec4.y,
                y:
                self.vec4.w * rhs.vec4.y
                    + self.vec4.y * rhs.vec4.w
                    + self.vec4.z * rhs.vec4.x 
                    - self.vec4.x * rhs.vec4.z,
                z:
                self.vec4.w * rhs.vec4.z
                    + self.vec4.z * rhs.vec4.w
                    + self.vec4.x * rhs.vec4.y
                    - self.vec4.y * rhs.vec4.x
            }
        }
    }
}



#[derive(Debug,Clone,Copy)]
#[repr(C)]
pub struct Matrix33{
    pub v1:Vec3,
    pub v2:Vec3,
    pub v3:Vec3
}

impl std::ops::Mul<Vec3> for Matrix33 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output{
        return Vec3{
            x:self.v1.x * rhs.x + self.v2.x * rhs.y + self.v3.x * rhs.z,
            y:self.v1.y * rhs.x + self.v2.y * rhs.y + self.v3.y * rhs.z,
            z:self.v1.z * rhs.x + self.v2.z * rhs.y + self.v3.z * rhs.z
        }
    }
}

impl std::ops::Add<Matrix33> for Matrix33 {
    type Output = Matrix33;

    fn add(self, rhs: Matrix33) -> Self::Output {
        return Matrix33 {
            v1:self.v1 + rhs.v1,
            v2:self.v2 + rhs.v2,
            v3:self.v3 + rhs.v3,
        }
    }
}

impl std::ops::Sub<Matrix33> for Matrix33 {
    type Output = Matrix33;

    fn sub(self, rhs: Matrix33) -> Self::Output {
        return Matrix33 {
            v1:self.v1 - rhs.v1,
            v2:self.v2 - rhs.v2,
            v3:self.v3 - rhs.v3,
        }
    }
}

impl std::ops::Mul<Matrix33> for Matrix33 {
    type Output = Matrix33;

    fn mul(self, rhs: Matrix33) -> Self::Output {
        return Matrix33 {
            v1:self * rhs.v1,
            v2:self * rhs.v2,
            v3:self * rhs.v3,
        }
    }
}

impl Matrix33 {
    pub fn transpose(&self) -> Matrix33 {
        return Matrix33 {
            v1:Vec3{
                x:self.v1.x,
                y:self.v2.x,
                z:self.v3.x
            },
            v2:Vec3{
                x:self.v1.y,
                y:self.v2.y,
                z:self.v3.y
            },
            v3:Vec3{
                x:self.v1.z,
                y:self.v2.z,
                z:self.v3.z
            }
        }
    }

    pub fn det(&self) -> f32{
        return (
            self.v1.x * self.v2.y * self.v3.z
                + self.v1.y * self.v2.z * self.v3.x
                + self.v1.z * self.v2.x * self.v3.y
        ) - (
            self.v1.z * self.v2.y * self.v3.x
                + self.v1.y * self.v2.x * self.v3.z
                + self.v1.x * self.v2.z * self.v3.y
        );
    }

    pub fn inverse(&self) -> Option<Matrix33>{
        let det = self.det();
        if det == 0.0{
            return None;
        }
        else{
            return Some(Matrix33{
                v1:Vec3{
                    x: (self.v2.y * self.v3.z - self.v2.z * self.v3.y) / det,
                    y: -(self.v1.y * self.v3.z - self.v1.z * self.v3.y) / det,
                    z: (self.v1.y * self.v2.z - self.v1.z * self.v2.y) / det
                },
                v2:Vec3{
                    x:-(self.v2.x * self.v3.z - self.v2.z * self.v3.x) / det,
                    y:(self.v1.x * self.v3.z - self.v1.z * self.v3.x) / det,
                    z:-(self.v1.x * self.v2.z - self.v1.z * self.v2.x) / det
                },
                v3:Vec3{
                    x:(self.v2.x * self.v3.y - self.v2.y * self.v3.x) / det,
                    y:-(self.v1.x * self.v3.y - self.v1.y * self.v3.x) / det,
                    z:(self.v1.x * self.v2.y - self.v1.y * self.v2.x) / det
                }
            })
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AABB3D{
    pub(crate) a: Vec3,
    pub(crate) b: Vec3,
}

impl AABB3D{
    pub fn generate_2(a: &Vec3, b: &Vec3) -> Self{
        return AABB3D{
            a: Vec3{
                x: f32::min(a.x, b.x),
                y: f32::min(a.y, b.y),
                z: f32::min(a.z, b.z),
            },
            b: Vec3{
                x: f32::max(a.x, b.x),
                y: f32::max(a.y, b.y),
                z: f32::max(a.z, b.z),
            },
        } 
    }
    
    pub fn generate_3(a: &Vec3, b: &Vec3, c: &Vec3) -> Self{
        return AABB3D{
            a: Vec3{
                x: f32::min(a.x, f32::min(b.x, c.x)),
                y: f32::min(a.y, f32::min(b.y, c.y)),
                z: f32::min(a.z, f32::min(b.z, c.z)),
            },
            b: Vec3{
                x: f32::max(a.x, f32::max(b.x, c.x)),
                y: f32::max(a.y, f32::max(b.y, c.y)),
                z: f32::max(a.z, f32::max(b.z, c.z)),
            },
        };
    }
    
    pub fn concat(lhs: &AABB3D, rhs: &AABB3D) -> AABB3D{
        return AABB3D{
            a: Vec3{
                x: f32::min(lhs.a.x, rhs.a.x),
                y: f32::min(lhs.a.y, rhs.a.y),
                z: f32::min(lhs.a.z, rhs.a.z),
            },
            b: Vec3{
                x: f32::max(lhs.b.x, rhs.b.x),
                y: f32::max(lhs.b.y, rhs.b.y),
                z: f32::max(lhs.b.z, rhs.b.z),
            },
        }
    }
}

