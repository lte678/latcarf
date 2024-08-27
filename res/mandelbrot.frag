out vec4 color;

uniform vec2 offset;
uniform float scale;

void main() {
    float z_real_sq = 0.0;
    float z_imag_sq = 0.0;
    float z_real = 0.0;
    float z_imag = 0.0;
    float z_prime_real = 1.0;
    float z_prime_imag = 0.0;
    float z_prime_rtmp;
    uint i = 0u;
    uint max_depth = 200u;
    while (i < max_depth && (z_real_sq + z_imag_sq) < 4.0) {
        z_prime_rtmp = z_prime_real;
        z_prime_real = 2.0*(z_real*z_prime_real - z_imag*z_prime_imag) + 1.0;
        z_prime_imag = 2.0*(z_real*z_prime_imag + z_imag*z_prime_rtmp);
        z_imag = 2.0*z_real*z_imag + (gl_FragCoord.y + offset.y)*scale;
        z_real = z_real_sq - z_imag_sq + (gl_FragCoord.x + offset.x)*scale;
        z_real_sq = z_real * z_real;
        z_imag_sq = z_imag * z_imag;
        i += 1u;
    }
    if(i == max_depth) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float z_mag = sqrt(z_real_sq + z_imag_sq);
        float z_prime_mag = sqrt(z_prime_real*z_prime_real + z_prime_imag*z_prime_imag);
        float dist = min((z_mag*log(z_mag)/z_prime_mag) / (0.15*scale), 1.0);
        color = vec4(0.0, 0.0, 0.0, 1.0) * (1.0-dist) + colormap(sqrt(float(i) / float(max_depth))) * dist;
    }
}
