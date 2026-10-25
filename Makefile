CXX=g++
CPPFLAGS+=-Dunix
UNAME_M:=$(shell uname -m)
UNAME_S:=$(shell uname -s)
# The JIT emits x86/x86-64 code, so other targets build with -DNOJIT.
ifneq ($(filter x86_64 amd64 i386 i486 i586 i686,$(UNAME_M)),)
ARCHFLAGS?=-march=native
else
CPPFLAGS+=-DNOJIT
ARCHFLAGS?=
endif
ifeq ($(UNAME_S),Darwin)
LTOFLAGS?=-flto
else
LTOFLAGS?=-flto=auto
endif
CXXFLAGS=-O3 $(ARCHFLAGS) $(LTOFLAGS) -funroll-loops
PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
MANDIR=$(PREFIX)/share/man

all: zpaq zpaq.1

libzpaq.o: libzpaq.cpp libzpaq.h
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -o $@ -c libzpaq.cpp

zpaq.o: zpaq.cpp libzpaq.h
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -o $@ -c zpaq.cpp -pthread

zpaq: zpaq.o libzpaq.o
	$(CXX) $(LDFLAGS) $(CXXFLAGS) -o $@ zpaq.o libzpaq.o -pthread

zpaq.1: zpaq.pod
	pod2man $< >$@

install: zpaq zpaq.1
	install -m 0755 -d $(DESTDIR)$(BINDIR)
	install -m 0755 zpaq $(DESTDIR)$(BINDIR)
	install -m 0755 -d $(DESTDIR)$(MANDIR)/man1
	install -m 0644 zpaq.1 $(DESTDIR)$(MANDIR)/man1

clean:
	rm -f zpaq.o libzpaq.o zpaq zpaq.1 archive.zpaq zpaq.new *.gcda *.gcno tests/sha1_test

check: zpaq
	./zpaq add archive.zpaq zpaq
	./zpaq extract archive.zpaq zpaq -to zpaq.new
	cmp zpaq zpaq.new
	rm archive.zpaq zpaq.new

tests/sha1_test: tests/sha1_test.cpp libzpaq.cpp libzpaq.h
	$(CXX) $(CPPFLAGS) -O2 $(ARCHFLAGS) -o $@ tests/sha1_test.cpp libzpaq.cpp

# Unit + round-trip tests. Set REF=/path/to/zpaq to also check archive
# compatibility in both directions against a reference build (e.g. 7.15).
test: zpaq tests/sha1_test
	./tests/sha1_test
	LIBZPAQ_NO_SHANI=1 ./tests/sha1_test
	./tests/fixtures.sh ./zpaq
	./tests/roundtrip.sh ./zpaq $(if $(REF),$(REF),)

pgo-generate:
	$(MAKE) clean
	rm -f *.gcda *.gcno
	$(MAKE) CXXFLAGS="$(CXXFLAGS) -fprofile-generate" LDFLAGS="$(LDFLAGS) -fprofile-generate" zpaq

pgo-run:
	./benchmark_zpaq.sh --quick

pgo-use:
	rm -f zpaq.o libzpaq.o zpaq
	$(MAKE) CXXFLAGS="$(CXXFLAGS) -fprofile-use -fprofile-correction" LDFLAGS="$(LDFLAGS) -fprofile-use" zpaq

pgo: pgo-generate pgo-run pgo-use

.PHONY: all install clean check test pgo-generate pgo-run pgo-use pgo
