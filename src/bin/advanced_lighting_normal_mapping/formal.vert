#version 330 core

// 点光源
struct PointLight {
    vec3 position;
};

in vec3 position;
in vec3 normal;
in vec2 tex_coords;
in vec3 tangent;
in vec3 bitangent;

// declare an interface block; see 'Advanced GLSL' for what these are.
out VS_OUT {
    vec3 FragPos;
    vec2 TexCoords;
    vec3 TangentLightPos;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
} vs_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform PointLight light;
uniform vec3 viewPos;

void main()
{
    vs_out.FragPos = vec3(model * vec4(position, 1.0));
    vs_out.TexCoords = tex_coords;

    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vec3 T = normalize(normalMatrix * tangent);
    vec3 N = normalize(normalMatrix * normal);
    T = normalize(T - dot(T, N) * N);
    vec3 B = cross(N, T);

    mat3 TBN = transpose(mat3(T, B, N));

    vs_out.TangentLightPos = TBN * light.position;
    vs_out.TangentViewPos  = TBN * viewPos;
    vs_out.TangentFragPos  = TBN * vs_out.FragPos;
    
    gl_Position = projection * view * model * vec4(position, 1.0);
}