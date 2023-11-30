use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use crate::ascia::core::{ColorRGBf32, Entity, ObjectNode};
use crate::ascia::math::{Vec3};

pub struct Light {
    pub objn: Rc<RefCell<ObjectNode>>,
    pub color: ColorRGBf32,
    pub power: f32
}

impl Light {
    pub fn new(color: ColorRGBf32, power:f32) -> Self{
        return Light {
            objn:ObjectNode::generate(),
            color:color,
            power:power
        }
    }
}

impl Entity for Light {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.objn;
    }
    fn update(&mut self,_d:&Duration) {

    }
}

impl Light {
    pub(crate) fn ray(&self, to: &Vec3) -> ColorRGBf32 {
        let distance = (*to - self.global_position()).norm();
        if distance == 0.0{
            return ColorRGBf32 {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }
        }
        //let brightness = self.power / (distance * distance);
        let brightness = self.power;
        return ColorRGBf32 {
            r: (self.color.r * brightness),
            g: (self.color.g * brightness),
            b: (self.color.b * brightness),
        }
    }
}