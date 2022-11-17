#version 330 core
                
in vec3 position;
in vec3 normal;
in vec2 texture;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    TexCoords = texture;    
    gl_Position = projection * view * model * vec4(position, 1.0);
}