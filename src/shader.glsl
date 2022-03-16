#version 450

#define PI 3.14159265358979323846264338327950288LF
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer Init {
    uint n;
    uint nsimx;
    uint nsimy;
    uint simz;
} init;

layout(set = 0, binding = 1) buffer Data {
    double angle[];
} buf;

layout(set = 0, binding = 2) buffer Data2 {
    int n[];
} buf2;

layout(set = 0, binding = 3) buffer Data3 {
    double len[];
} buf3;

//G.P.U.s are bad at random
double random (vec2 st) {
    return fract(sin(dot(st.xy,
    vec2(PI/2,78.233LF)))*
    43758.543123LF);
}

void main() {
    uint idx = gl_GlobalInvocationID.x+gl_GlobalInvocationID.y*init.nsimx+gl_GlobalInvocationID.z*init.nsimx*init.nsimy;
    vec2 l=vec2(0.0LF,0.0LF);
    int Nn=0;
    int Np=0;

    vec2 d=vec2(cos(float(buf.angle[idx*init.n])),sin(float(buf.angle[idx*init.n])));
    l+=d;

    for (int i = 1;i<init.n;i++){

        buf.angle[idx*2+((i)%2)]=random(mat2x2(cos(float(buf.angle[idx*2+((i-1)%2)])),
        sin(float(buf.angle[idx*2+((i-1)%2)])),
        -sin(float(buf.angle[idx*2+((i-1)%2)])),
        cos(float(buf.angle[idx*2+((i-1)%2)])))*l*2)*PI*2-PI;
        vec2 d=vec2(cos(float(buf.angle[idx*2+((i)%2)])),sin(float(buf.angle[idx*2+((i)%2)])));
        l+=d;
        double a=abs(buf.angle[idx*2+(i%2)]-buf.angle[idx*2+((i-1)%2)]);
        if (a<-PI){
            a+=2*PI;
        }
        if (a>PI){
            a-=2*PI;
        }
        if (abs(a)<PI/2){
            Nn+=1;
        } else {
            Np+=1;
        }
    }

    buf3.len[idx]=sqrt(l.x*l.x+l.y*l.y);
    buf2.n[idx]=Nn-Np;
}
