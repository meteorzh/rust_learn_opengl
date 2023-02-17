#version 330 core
in vec3 position;
in vec3 normal;
in vec2 texture;

out vec3 FragPos;
out vec2 TexCoords;
out vec3 Normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vec4 worldPos = model * vec4(position, 1.0);
    FragPos = worldPos.xyz; 
    TexCoords = texture;
    
    mat3 normalMatrix = transpose(inverse(mat3(model)));
    Normal = normalMatrix * normal;

    gl_Position = projection * view * worldPos;
}