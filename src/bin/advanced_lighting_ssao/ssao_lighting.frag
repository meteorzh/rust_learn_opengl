#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D GPosition;
uniform sampler2D GNormal;
uniform sampler2D GColorSpecular;
uniform sampler2D ssao;

struct Light {
    vec3 position;
    vec3 color;
    
    float linear;
    float quadratic;
};
uniform Light light;

void main()
{             
    // retrieve data from gbuffer
    vec3 FragPos = texture(GPosition, TexCoords).rgb;
    vec3 Normal = texture(GNormal, TexCoords).rgb;
    vec3 Diffuse = texture(GColorSpecular, TexCoords).rgb;
    float AmbientOcclusion = texture(ssao, TexCoords).r;
    
    // then calculate lighting as usual
    vec3 ambient = vec3(0.3 * Diffuse * AmbientOcclusion);
    vec3 lighting  = ambient; 
    vec3 viewDir  = normalize(-FragPos); // viewpos is (0.0.0)
    // diffuse
    vec3 lightDir = normalize(light.position - FragPos);
    vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Diffuse * light.color;
    // specular
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    float spec = pow(max(dot(Normal, halfwayDir), 0.0), 8.0);
    vec3 specular = light.color * spec;
    // attenuation
    float distance = length(light.position - FragPos);
    float attenuation = 1.0 / (1.0 + light.linear * distance + light.quadratic * distance * distance);
    diffuse *= attenuation;
    specular *= attenuation;
    lighting += diffuse + specular;

    FragColor = vec4(lighting, 1.0);
}
