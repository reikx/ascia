use std::cell::RefCell;
use std::rc::Rc;
use crate::ascia::color::ColorRGBf32;
use crate::ascia::core::{AsciaEnvironment, Global, Light, ObjectNode, ObjectNodeAttribute, PresetAsciaEnvironment, PresetLight, PresetObjectNodeAttribute};
use crate::ascia::math::{Vec3};

pub struct PointLight {
    pub color: ColorRGBf32,
    pub power: f32
}

impl ObjectNodeAttribute<PresetAsciaEnvironment> for PointLight {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Light(PresetLight::Point(self))));
    }
}

impl<E:AsciaEnvironment> Light<E> for PointLight where PointLight: ObjectNodeAttribute<E>{
    fn ray(&self, node:&ObjectNode<Global, E>, to: &Vec3) -> ColorRGBf32 {
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