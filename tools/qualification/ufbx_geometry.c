/* Offline independent FBX geometry/axis probe. Link only in ignored build/.
 * Reference: ufbx 0.23.1, commit 26a482ae66871d7de36eb722aa060bce95bce274.
 * This is a verifier, not a client/server loader or Unity importer substitute. */
#include "ufbx.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static void string(ufbx_string s) {
    putchar('"');
    for (size_t i=0;i<s.length;i++) {
        unsigned char c=(unsigned char)s.data[i];
        if(c=='"'||c=='\\') {putchar('\\');putchar(c);}
        else if(c<32) printf("\\u%04x",c);
        else putchar(c);
    }
    putchar('"');
}
static void vector(ufbx_vec3 v) {printf("[%.17g,%.17g,%.17g]",v.x,v.y,v.z);}
static void matrix(ufbx_matrix m) {
    printf("[[%.17g,%.17g,%.17g,%.17g],[%.17g,%.17g,%.17g,%.17g],[%.17g,%.17g,%.17g,%.17g],[0,0,0,1]]",
        m.m00,m.m01,m.m02,m.m03,m.m10,m.m11,m.m12,m.m13,m.m20,m.m21,m.m22,m.m23);
}
int main(int argc,char **argv) {
    if(argc!=3) {fprintf(stderr,"usage: ufbx_geometry model.fbx right|mirror-x|left\n");return 2;}
    FILE *input=fopen(argv[1],"rb");
    if(!input) {fprintf(stderr,"input open failed\n");return 1;}
    if(fseek(input,0,SEEK_END)!=0) {fclose(input);return 1;}
    long size=ftell(input);fclose(input);
    if(size<27||size>128L*1024L*1024L) {fprintf(stderr,"input byte budget exceeded\n");return 1;}
    ufbx_load_opts opts={0};
    opts.file_format=UFBX_FILE_FORMAT_FBX;
    opts.target_axes=ufbx_axes_right_handed_y_up;
    if(!strcmp(argv[2],"mirror-x")) opts.target_axes.right=UFBX_COORDINATE_AXIS_NEGATIVE_X;
    else if(!strcmp(argv[2],"left")) opts.target_axes=ufbx_axes_left_handed_y_up;
    else if(strcmp(argv[2],"right")) {fprintf(stderr,"invalid coordinate profile\n");return 2;}
    opts.target_unit_meters=1.;
    opts.space_conversion=UFBX_SPACE_CONVERSION_MODIFY_GEOMETRY;
    opts.handedness_conversion_axis=UFBX_MIRROR_AXIS_X;
    opts.ignore_animation=true;opts.ignore_embedded=true;opts.load_external_files=false;
    opts.temp_allocator.memory_limit=512u*1024u*1024u;
    opts.result_allocator.memory_limit=512u*1024u*1024u;
    opts.node_depth_limit=128;
    ufbx_error error={0};ufbx_scene *scene=ufbx_load_file(argv[1],&opts,&error);
    if(!scene) {fprintf(stderr,"ufbx error: %.*s\n",(int)error.description.length,error.description.data);return 1;}
    if(scene->nodes.count>65536) {fprintf(stderr,"node budget exceeded\n");ufbx_free_scene(scene);return 1;}
    printf("{\"ufbx_version\":%u,\"source_unit_meters\":%.17g,\"source_axes\":[%d,%d,%d],\"nodes\":[",
        ufbx_source_version,scene->settings.unit_meters,scene->settings.axes.right,scene->settings.axes.up,scene->settings.axes.front);
    size_t count=0,total=0;
    for(size_t ni=0;ni<scene->nodes.count;ni++) {
        ufbx_node *node=scene->nodes.data[ni];ufbx_mesh *mesh=node->mesh;if(!mesh) continue;
        if(mesh->num_vertices>2000000||mesh->num_indices>8000000||total>8000000-mesh->num_vertices) {fprintf(stderr,"vertex budget exceeded\n");ufbx_free_scene(scene);return 1;}
        total+=mesh->num_vertices;
        if(count++) putchar(',');
        printf("{\"name\":");string(node->name);
        printf(",\"deformed\":%s,\"node_to_world\":",(mesh->skin_deformers.count||mesh->blend_deformers.count)?"true":"false");matrix(node->node_to_world);
        printf(",\"geometry_to_node\":");matrix(node->geometry_to_node);
        printf(",\"geometry_to_world\":");matrix(node->geometry_to_world);
        printf(",\"vertices\":[");
        for(size_t vi=0;vi<mesh->vertices.count;vi++) {if(vi)putchar(',');vector(mesh->vertices.data[vi]);}
        printf("],\"faces\":[");
        for(size_t fi=0;fi<mesh->faces.count;fi++) {
            if(fi) {putchar(',');}
            putchar('[');ufbx_face face=mesh->faces.data[fi];
            for(uint32_t j=0;j<face.num_indices;j++) {if(j)putchar(',');printf("%u",mesh->vertex_indices.data[face.index_begin+j]);}
            putchar(']');
        }
        printf("]}");
    }
    printf("]}\n");ufbx_free_scene(scene);return ferror(stdout)?1:0;
}
