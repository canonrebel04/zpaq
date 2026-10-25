// SHA-1 known-answer and consistency test for libzpaq::SHA1.
// Build: g++ -O2 -Dunix tests/sha1_test.cpp libzpaq.cpp -o sha1_test
// Run it twice, with and without LIBZPAQ_NO_SHANI=1, to test both paths.
#include "../libzpaq.h"
#include <stdio.h>
#include <string.h>
#include <string>

static std::string hex(const char* h) {
  std::string s; char b[3];
  for (int i=0; i<20; ++i) snprintf(b, 3, "%02x", h[i]&255), s+=b;
  return s;
}

int main() {
  int fail=0;
  struct { const char* in; long rep; const char* out; } kat[]={
    {"", 1, "da39a3ee5e6b4b0d3255bfef95601890afd80709"},
    {"abc", 1, "a9993e364706816aba3e25717850c26c9cd0d89d"},
    {"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", 1,
     "84983e441c3bd26ebaae4aa1f95129e5e54670f1"},
    {"a", 1000000, "34aa973cd4c4daa4f61eeb2bdbad27316534016f"},
  };
  for (auto& k: kat) {
    libzpaq::SHA1 s;
    std::string in;
    for (long i=0; i<k.rep; ++i) in+=k.in;
    s.write(in.data(), in.size());
    std::string r=hex(s.result());
    if (r!=k.out) printf("FAIL write %s\n", k.out), ++fail;
    for (size_t i=0; i<in.size(); ++i) s.put(in[i]);
    r=hex(s.result());
    if (r!=k.out) printf("FAIL put %s\n", k.out), ++fail;
  }
  // unaligned mixes of put() and write() must match one-shot write()
  std::string data;
  unsigned x=12345;
  for (int i=0; i<100000; ++i) x=x*1103515245+12345, data+=char(x>>16);
  libzpaq::SHA1 a, b;
  a.write(data.data(), data.size());
  std::string ra=hex(a.result());
  size_t p=0;
  for (int n=1; p<data.size(); n=n*7%251+1) {
    size_t m=std::min(size_t(n), data.size()-p);
    if (n&1) b.write(data.data()+p, m);
    else for (size_t i=0; i<m; ++i) b.put(data[p+i]);
    p+=m;
  }
  if (hex(b.result())!=ra) printf("FAIL mixed put/write\n"), ++fail;
  printf("%s sha1 %s\n", fail ? "FAIL" : "ok", ra.c_str());
  return fail!=0;
}
