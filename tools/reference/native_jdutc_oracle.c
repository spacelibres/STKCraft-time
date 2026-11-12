/* Independent N0067 UNITIM/TTRANS capture; never linked into production. */
#include "SpiceUsr.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int ttrans_(char *, char *, SpiceDouble *, SpiceInt, SpiceInt);
static double number(const char *text) {
    uint64_t word=strtoull(text,NULL,16);double value;memcpy(&value,&word,8);return value;
}
static void bits(double value) {
    uint64_t word;memcpy(&word,&value,8);printf("%016llx",(unsigned long long)word);
}
static int error(void) {
    char short_message[256],long_message[2048];
    if(!failed_c())return 0;
    getmsg_c("SHORT",sizeof(short_message),short_message);
    getmsg_c("LONG",sizeof(long_message),long_message);
    printf("|E|%s\n",short_message);
    fprintf(stderr,"NATIVE_ERROR|%s|%s\n",short_message,long_message);
    reset_c();return 1;
}
int main(void) {
    char line[65536],original[65536],profile[128]="",*context,*token;
    const char *names[]={"DELTET/DELTA_T_A","DELTET/K","DELTET/EB","DELTET/M"};
    fprintf(stderr,"CRT_FMA:%d\n",_get_FMA3_enable());
    if(_get_FMA3_enable()!=1)return 1;
    erract_c("SET",0,"RETURN");errprt_c("SET",0,"NONE");
    while(fgets(line,sizeof(line),stdin)) {
        line[strcspn(line,"\r\n")]=0;strcpy(original,line);
        token=strtok_s(line,"|",&context);
        if(!token)return 2;
        if(strcmp(token,"C")==0) {
            SpiceDouble constants[5],entries[600];SpiceInt count,i;
            kclear_c();strcpy(profile,strtok_s(NULL,"|",&context));
            count=atoi(strtok_s(NULL,"|",&context));if(count<1||count>300)return 3;
            for(i=0;i<5;++i)constants[i]=number(strtok_s(NULL,"|",&context));
            for(i=0;i<count;++i) {
                double jdn=(double)_strtoi64(strtok_s(NULL,"|",&context),NULL,10);
                entries[2*i]=(double)strtol(strtok_s(NULL,"|",&context),NULL,10);
                entries[2*i+1]=(jdn-2451545.0)*86400.0-43200.0;
            }
            for(i=0;i<3;++i)pdpool_c(names[i],1,&constants[i]);
            pdpool_c(names[3],2,&constants[3]);pdpool_c("DELTET/DELTA_AT",count*2,entries);
            printf("%s",original);if(!error())puts("|OK");
        } else if(strcmp(token,"Q")==0 || strcmp(token,"B")==0) {
            double value=number(strtok_s(NULL,"|",&context)),et,vector[7]={0};
            et=strcmp(token,"B")==0 ? unitim_c(value,"TAI","TDB") : value;
            printf("R|%s|",profile);bits(et);
            if(error())continue;
            vector[0]=et;ttrans_("TDB","JDUTC",vector,3,5);
            if(!error()){printf("|N|");bits(vector[0]);putchar('\n');}
        } else return 4;
    }
    kclear_c();return 0;
}
