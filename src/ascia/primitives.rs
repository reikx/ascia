use crate::ascia::core::{AsciaEnvironment, Local, Polygon, PresetMaterial};
use crate::ascia::math::{Matrix33, Vec3};

pub struct PrimitiveGenerator{
    
}

impl PrimitiveGenerator {
    pub fn square<E: AsciaEnvironment>(size:f32, material: E::Materials) -> Vec<Polygon<Local, E>>{
        let mut polygons = vec![];
        let p = size * 0.5;
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:0.0, y:p,z:-p},
                v2:Vec3{ x:0.0, y:-p, z:-p},
                v3:Vec3{ x:0.0, y:p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:0.0, y:-p,z:p},
                v2:Vec3{ x:0.0, y:p, z:p},
                v3:Vec3{ x:0.0, y:-p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        return polygons;
    }

    pub fn cube<E: AsciaEnvironment>(size:f32,material: E::Materials) -> Vec<Polygon<Local, E>>{
        let mut polygons = vec![];
        let p = size * 0.5;

        polygons.push(Polygon {
            poses:Matrix33 {
                v1: Vec3 { x: -p, y: p, z: -p },
                v2: Vec3 { x: -p, y: -p, z: -p },
                v3: Vec3 { x: -p, y: p, z: p }
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:p},
                v2:Vec3{ x:-p, y:p, z:p},
                v3:Vec3{ x:-p, y:-p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:p},
                v2:Vec3{ x:p, y:-p, z:p},
                v3:Vec3{ x:p, y:p, z:-p},
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v3:Vec3{ x:p, y:-p, z:-p},
                v1:Vec3{ x:p, y:p,z:-p},
                v2:Vec3{ x:p, y:-p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });

        
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:p},
                v2:Vec3{ x:-p, y:-p, z:-p},
                v3:Vec3{ x:p, y:-p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:-p,z:-p},
                v2:Vec3{ x:p, y:-p, z:p},
                v3:Vec3{ x:-p, y:-p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:p,z:-p},
                v2:Vec3{ x:-p, y:p, z:p},
                v3:Vec3{ x:p, y:p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:p},
                v2:Vec3{ x:p, y:p, z:-p},
                v3:Vec3{ x:-p, y:p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });


        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:p,z:-p},
                v2:Vec3{ x:p, y:-p, z:-p},
                v3:Vec3{ x:-p, y:p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:-p,z:-p},
                v2:Vec3{ x:-p, y:p, z:-p},
                v3:Vec3{ x:p, y:-p, z:-p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });

        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:-p, y:p,z:p},
                v2:Vec3{ x:-p, y:-p, z:p},
                v3:Vec3{ x:p, y:p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });
        polygons.push(Polygon {
            poses:Matrix33{
                v1:Vec3{ x:p, y:-p,z:p},
                v2:Vec3{ x:p, y:p, z:p},
                v3:Vec3{ x:-p, y:-p, z:p}
            },
            material:material.clone(),
            _ph: Default::default(),
        });

        return polygons;
    }
}