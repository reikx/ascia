use crate::ascia::core::{Material, Polygon};
use crate::ascia::math::{Matrix33, Vec3};

pub struct PrimitiveGenerator{

}

impl PrimitiveGenerator {
    pub fn square(size:f32,material:Material) -> Vec<Polygon>{
        let mut polygons = vec![];
        let p = size * 0.5;
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:0.0, y:p,z:-p},
                v2:Vec3{ x:0.0, y:-p, z:-p},
                v3:Vec3{ x:0.0, y:p, z:p}
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:0.0, y:-p,z:p},
                v2:Vec3{ x:0.0, y:p, z:p},
                v3:Vec3{ x:0.0, y:-p, z:-p}
            },
            material:material.clone()
        });
        return polygons;
    }

    pub fn cube(size:f32,material:Material) -> Vec<Polygon>{
        let mut polygons = vec![];
        let p = size * 0.5;

        polygons.push(Polygon {
            poses:Matrix33 {
                v1: Vec3 { x: -p, y: p, z: -p },
                v2: Vec3 { x: -p, y: -p, z: -p },
                v3: Vec3 { x: -p, y: p, z: p }
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:p},
                v2:Vec3{ x:-p, y:p, z:p},
                v3:Vec3{ x:-p, y:-p, z:-p}
            },
            material:material.clone()
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:p},
                v2:Vec3{ x:p, y:-p, z:p},
                v3:Vec3{ x:p, y:p, z:-p},
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v3:Vec3{ x:p, y:-p, z:-p},
                v1:Vec3{ x:p, y:p,z:-p},
                v2:Vec3{ x:p, y:-p, z:p}
            },
            material:material.clone()
        });



        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:p},
                v2:Vec3{ x:-p, y:-p, z:-p},
                v3:Vec3{ x:p, y:-p, z:p}
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:-p,z:-p},
                v2:Vec3{ x:p, y:-p, z:p},
                v3:Vec3{ x:-p, y:-p, z:-p}
            },
            material:material.clone()
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:p,z:-p},
                v2:Vec3{ x:-p, y:p, z:p},
                v3:Vec3{ x:p, y:p, z:-p}
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:p},
                v2:Vec3{ x:p, y:p, z:-p},
                v3:Vec3{ x:-p, y:p, z:p}
            },
            material:material.clone()
        });


        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:-p},
                v2:Vec3{ x:p, y:-p, z:-p},
                v3:Vec3{ x:-p, y:p, z:-p}
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:-p},
                v2:Vec3{ x:-p, y:p, z:-p},
                v3:Vec3{ x:p, y:-p, z:-p}
            },
            material:material.clone()
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:p,z:p},
                v2:Vec3{ x:-p, y:-p, z:p},
                v3:Vec3{ x:p, y:p, z:p}
            },
            material:material.clone()
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:-p,z:p},
                v2:Vec3{ x:p, y:p, z:p},
                v3:Vec3{ x:-p, y:-p, z:p}
            },
            material:material.clone()
        });

        return polygons;
    }
}