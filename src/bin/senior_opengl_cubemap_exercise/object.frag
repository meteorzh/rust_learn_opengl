#version 330 core

// 材质
struct Material {
    sampler2D diffuse;
    sampler2D reflection;
    samplerCube skybox;
};

out vec4 FragColor;

in vec3 Normal;
in vec2 TexCoords;
in vec3 Position;

uniform vec3 viewPos;
uniform Material material;

void main()
{
    vec3 viewDir = normalize(viewPos - Position);
    vec3 normal = normalize(Normal);

    vec3 R = reflect(-viewDir, normal);
    vec3 reflectMap = vec3(texture(material.reflection, TexCoords));
    vec3 reflection = vec3(texture(material.skybox, R).rgb) * reflectMap;

    float diff = max(normalize(dot(normal, viewDir)), 0.0f);
    vec3 diffuse = diff * vec3(texture(material.diffuse, TexCoords));

    FragColor = vec4(diffuse + reflection, 1.0f);
}