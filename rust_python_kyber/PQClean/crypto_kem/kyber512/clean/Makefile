# This Makefile can be used with GNU Make or BSD Make

LIB=libkyber512_clean.a
HEADERS=api.h cbd.h indcpa.h kem.h ntt.h params.h poly.h polyvec.h reduce.h symmetric.h verify.h 
OBJECTS=cbd.o indcpa.o kem.o ntt.o poly.o polyvec.o reduce.o symmetric-shake.o verify.o 

COMMON=../../../common
COMMON_OBJECTS=$(patsubst %.c,%.o,$(wildcard $(COMMON)/*.c))

CFLAGS=-O3 -fPIC -std=c99 -I../../../common $(EXTRAFLAGS)

$(COMMON)/%.o: $(COMMON)/%.c $(COMMON)/%.h
	$(CC) $(CFLAGS) -c -o $@ $<

all: $(LIB)

%.o: %.c $(HEADERS)
	$(CC) $(CFLAGS) -c -o $@ $<


$(LIB): $(OBJECTS) $(COMMON_OBJECTS)
	$(AR) -r $@ $(OBJECTS) $(COMMON_OBJECTS)

clean:
	$(RM) $(OBJECTS)
	$(RM) $(LIB)
