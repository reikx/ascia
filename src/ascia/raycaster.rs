use crate::ascia::core::{Polygon};
use crate::ascia::math::{Matrix33, Vec2, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct RaycasterIntersection{
    pub polygon_index:usize,
    pub depth:f32,
    pub intersection_position_on_polygon:Vec2,
    pub intersection_position_global:Vec3
}

pub struct Raycaster{

}

impl Raycaster{
    #[inline]
    pub fn project_polygon(global_position:&Vec3, global_direction:&Vec3, polygon_index:usize, global_polygons:&[Polygon]) -> Option<RaycasterIntersection>{
        let m1 = global_polygons[polygon_index];
        let m2 = Matrix33{
            v1:m1.poses.v1 - *global_position,
            v2:m1.poses.v2 - *global_position,
            v3:m1.poses.v3 - *global_position,
        };
        if let Some(m3) = m2.inverse(){
            let v1 = m3 * (*global_direction);
            let psy = 1.0 / (v1.x + v1.y + v1.z);
            if psy != f32::INFINITY && 0.0 < psy{
                let v2 = v1 * psy;
                if (0.0 <= v2.x && v2.x <= 1.0) && (0.0 <= v2.y && v2.y <= 1.0) && (0.0 <= v2.z && v2.z <= 1.0){
                    let depth = psy * global_direction.norm();
                    if depth > 0.0{
                        return Some(RaycasterIntersection{
                            polygon_index: polygon_index,
                            intersection_position_on_polygon: Vec2{
                                x:v2.y,
                                y:v2.z
                            },
                            depth: depth,
                            intersection_position_global: m1.poses.v1 + v2.y * (m1.poses.v2 - m1.poses.v1) + v2.z * (m1.poses.v3 - m1.poses.v1),
                        });
                    }
                }
            }
        }
        return None;
    }


}

