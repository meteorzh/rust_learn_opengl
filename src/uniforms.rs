use std::collections::HashMap;

use glium::uniforms::{UniformValue, AsUniformValue, Uniforms};

#[macro_export]
macro_rules! dynamic_uniform{
    () => {
        $crate::uniforms::DynamicUniforms::new()
    };

    ($($field:ident: $value:expr), *,) => {
        {
            let mut tmp = $crate::DynamicUniforms::new();
            $(
                tmp.add(stringify!($field), $value);
            )*
            tmp
        }
    };
}

#[derive(Clone)]
pub struct DynamicUniforms<'a>{
    map: HashMap<String, UniformValue<'a>>,
}

impl<'a> DynamicUniforms<'a>{
    /// Creates new DynamicUniforms
    pub fn new() -> Self{
        Self{
            map: HashMap::new()
        }
    }

    /// Add a value to the DynamicUniforms
    #[inline]
    pub fn add(&mut self, key: String, value: &'a dyn AsUniformValue) {
        self.map.insert(key, value.as_uniform_value());
    }

    /// Add a value to the DynamicUniforms
    #[inline]
    pub fn add_str_key(&mut self, key: &str, value: &'a dyn AsUniformValue) {
        self.map.insert(String::from(key), value.as_uniform_value());
    }

    /// Add a value to the DynamicUniforms
    #[inline]
    pub fn add_str_key_value(&mut self, key: &str, value: UniformValue<'a>) {
        self.map.insert(String::from(key), value);
    }

    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl Uniforms for DynamicUniforms<'_>{
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        for (key, value) in self.map.iter(){
            output(key, *value);
        }
    }
}


/**
 * 将指定值按指定key添加到uniforms
 */
pub fn add_to_uniforms<'a: 'b, 'b>(key_prefix: &str, key_suffix: &str, value: &'a dyn AsUniformValue, uniforms: &'b mut DynamicUniforms<'a>) {
    let mut key = String::from(key_prefix);
    key.push_str(key_suffix);
    uniforms.add(key, value);
}