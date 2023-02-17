#version 330 core

// 点光源
struct PointLight {
    vec3 position;
    vec3 color;
};

#define NR_POINT_LIGHTS 4

out vec4 FragColor;

in VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
} fs_in;

uniform sampler2D floorTexture;
uniform PointLight lights[NR_POINT_LIGHTS];
uniform vec3 viewPos;
uniform bool gamma;

vec3 BlinnPhong(vec3 normal, vec3 fragPos, PointLight light)
{
    // diffuse
    vec3 lightDir = normalize(light.position - fragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * light.color;
    // specular
    vec3 viewDir = normalize(viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = 0.0;
    vec3 halfwayDir = normalize(lightDir + viewDir);
    spec = pow(max(dot(normal, halfwayDir), 0.0), 64.0);
    vec3 specular = spec * light.color;
    // simple attenuation
    float max_distance = 1.5;
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (gamma ? distance * distance : distance);
    
    diffuse *= attenuation;
    specular *= attenuation;
    
    return diffuse + specular;
}

void main()
{           
    vec3 color = texture(floorTexture, fs_in.TexCoords).rgb;
    vec3 lighting = vec3(0.0);
    for(int i = 0; i < NR_POINT_LIGHTS; ++i)
        lighting += BlinnPhong(normalize(fs_in.Normal), fs_in.FragPos, lights[i]);
    color *= lighting;
    if(gamma)
        color = pow(color, vec3(1.0/2.2));
    FragColor = vec4(color, 1.0);
}