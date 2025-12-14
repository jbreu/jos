#define wchar_t long
#define wctype_t int
#define wint_t long

wctype_t wctype(const char *property);
int iswctype(wint_t wc, wctype_t desc);