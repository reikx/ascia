use std::cell::RefCell;
use std::rc::Rc;
use crate::ascia::color::ColorRGBf32;
use crate::ascia::core::{Global, Light, ObjectNode, ObjectNodeAttribute, PresetLight, PresetObjectNodeAttribute};
use crate::ascia::math::{Vec3};

pub struct PointLight {
    pub color: ColorRGBf32,
    pub power: f32
}

impl ObjectNodeAttribute for PointLight {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Light(PresetLight::Point(self))));
    }
}

impl Light for PointLight{
    fn ray(&self, node:&ObjectNode<Global>, to: &Vec3) -> ColorRGBf32 {
        let distance = (*to - node.position).norm();
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
/*
impl Light {
    pub fn new(color: ColorRGBf32, power:f32) -> Self{
        return Light {
            objn:ObjectNode::generate(),
            color:color,
            power:power
        }
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
} */