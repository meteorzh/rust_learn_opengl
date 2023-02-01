#version 330 core

// 点光源
struct PointLight {
    vec3 position;
    vec3 color;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

out vec4 FragColor;

uniform PointLight light;

void main() {
    FragColor = vec4(light.color, 1.0);
}