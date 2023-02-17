#version 330 core
in vec3 position;
in vec3 normal;
in vec2 texture;

out vec3 FragPos;
out vec2 TexCoords;
out vec3 Normal;

uniform bool invertedNormals;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vec4 viewPos = view * model * vec4(position, 1.0);
    FragPos = viewPos.xyz; 
    TexCoords = texture;
    
    mat3 normalMatrix = transpose(inverse(mat3(view * model)));
    Normal = normalMatrix * (invertedNormals ? -normal : normal);
    
    gl_Position = projection * viewPos;
}