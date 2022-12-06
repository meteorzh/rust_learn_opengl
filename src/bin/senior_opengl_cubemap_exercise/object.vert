#version 330 core
in vec3 position;
in vec3 normal;
in vec2 texture;

out vec3 Normal;
out vec2 TexCoords;
out vec3 Position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    Normal = mat3(transpose(inverse(model))) * normal;
    Position = vec3(model * vec4(position, 1.0));
    gl_Position = projection * view * model * vec4(position, 1.0);
    TexCoords = texture;
}