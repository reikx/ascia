use crate::ascia::color::ColorRGBf32;
use crate::ascia::core::{AsciaEnvironment, Global, Light, LightDispatcher, ObjectNode, ObjectNodeAttribute, PresetLight, PresetObjectNodeAttributeDispatcher};
use crate::ascia::math::{Vec3};

pub struct PointLight{
    pub color: ColorRGBf32,
    pub power: f32,
}

impl Default for PointLight{
    fn default() -> Self {
        PointLight{
            color: ColorRGBf32{
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            power: 1.0,
        }
    }
}

impl<E: AsciaEnvironment<ObjectNodeAttributes = PresetObjectNodeAttributeDispatcher<E>, Lights = PresetLight>> From<PointLight> for PresetObjectNodeAttributeDispatcher<E>{
    fn from(value: PointLight) -> PresetObjectNodeAttributeDispatcher<E> {
        <PresetLight as LightDispatcher<E>>::make_attribute_enum(PresetLight::PointLight(value))
    }
}

impl<E: AsciaEnvironment> ObjectNodeAttribute<E> for PointLight{}

impl<E: AsciaEnvironment> Light<E> for PointLight{
    fn ray(&self, node:&ObjectNode<E, Global>, to: &Vec3) -> ColorRGBf32 {
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