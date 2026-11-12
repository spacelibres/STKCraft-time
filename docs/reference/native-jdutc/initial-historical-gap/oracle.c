/* Independent N0067 historical UTC contract probe; no production linkage. */
#include "SpiceUsr.h"
#include <stdint.h>
#include <stdio.h>
#include <string.h>
extern int ttrans_(char *, char *, SpiceDouble *, SpiceInt, SpiceInt);
static void bits(double value) {
    uint64_t encoded; memcpy(&encoded,&value,sizeof(encoded));
    printf("%016llx",(unsigned long long)encoded);
}
int main(void) {
    const double epochs[]={-4000000000.,-2000000000.,-1500000000.,-1000000000.,
        -900000000.,-883656100.,-883656010.,-883656000.,-883655958.,
        -883655957.816,-883655957.,-883655900.,0.};
    int i; char utc[128],message[256];
    fprintf(stderr,"CRT_FMA:%d\n",_get_FMA3_enable());
    if(_get_FMA3_enable()!=1)return 1;
    erract_c("SET",0,"RETURN");errprt_c("SET",0,"NONE");
    furnsh_c("target/kernels/naif0012.tls");
    if(failed_c())return 2;
    for(i=0;i<(int)(sizeof(epochs)/sizeof(epochs[0]));++i){
        SpiceDouble vector[7]={0},delta;
        bits(epochs[i]); putchar('|');
        vector[0]=epochs[i];ttrans_("TDB","JDUTC",vector,3,5);
        if(!failed_c())deltet_c(epochs[i],"ET",&delta);
        if(!failed_c())et2utc_c(epochs[i],"ISOC",14,sizeof(utc),utc);
        if(failed_c()){
            getmsg_c("SHORT",sizeof(message),message);printf("E|%s\n",message);reset_c();continue;
        }
        bits(vector[0]);putchar('|');bits(delta);printf("|%s\n",utc);
    }
    return 0;
}
