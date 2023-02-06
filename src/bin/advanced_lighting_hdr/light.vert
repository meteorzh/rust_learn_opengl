#version 330 core
in vec3 position;
in vec3 normal;
in vec2 texture;

out VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
} vs_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform bool inverse_normals;

void main()
{
    vs_out.FragPos = vec3(model * vec4(position, 1.0));   
    vs_out.TexCoords = texture;
    
    vec3 n = inverse_normals ? -normal : normal;
    
    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vs_out.Normal = normalize(normalMatrix * n);
    
    gl_Position = projection * view * model * vec4(position, 1.0);
}