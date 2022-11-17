#version 330 core
                
in vec3 position;
in vec3 normal;
in vec2 texture;

out vec3 oNormal;
out vec3 FragPos;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    FragPos = vec3(model * vec4(position, 1.0));
    oNormal = mat3(transpose(inverse(model))) * normal;
    // gl_Position = projection * view * vec4(FragPos, 1.0);

    TexCoords = texture;
}