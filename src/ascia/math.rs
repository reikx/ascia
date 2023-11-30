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
    pub(crate) fn norm(&self) -> f32{
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


#[derive(Debug,Clone,Copy)]
#[repr(C,align(16))]
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
    pub(crate) fn norm(&self) -> f32{
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

#[derive(Debug,Clone,Copy)]
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


#[derive(Debug,Clone,Copy)]
#[repr(C)]
pub struct Quaternion {
    pub vec4:Vec4
}

impl Quaternion{
    pub fn new(axis:Vec3,deg:f32) -> Self{
        let s = f32::sin(deg / 2.0);
        let c = f32::cos(deg / 2.0);
        return Quaternion{
            vec4:Vec4{
                w:c,
                x:s * axis.x,
                y:s * axis.y,
                z:s * axis.z,
            }
        }
    }

    pub fn rotate(&self,v:&Vec3) -> Vec3{
        let c = self.conjugate();
        let res = (*self * Quaternion{
            vec4:Vec4{
                w:0.0,
                x:v.x,
                y:v.y,
                z:v.z
            }
        }) * c;
        return Vec3{
            x:res.vec4.x,
            y:res.vec4.y,
            z:res.vec4.z
        }
    }

    pub fn rotator(from:&Vec3,to:&Vec3) -> Self{
        let axis = (*from ^ *to).normalize();
        return Quaternion::new(axis,f32::acos((*from * *to) / (from.norm() * to.norm())));
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
                    - self.vec4.x * rhs.vec4.z
                    + self.vec4.z * rhs.vec4.x,
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