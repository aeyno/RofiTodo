AC_INIT([RofiTodo], m4_esyscmd([grep ^version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n"]), [tom.gouville@protonmail.com])
AM_INIT_AUTOMAKE([1.11 foreign no-dependencies no-dist-gzip dist-xz subdir-objects])

VERSION=$(grep ^version Cargo.toml | awk '{print $3}' | tr -d '"' | tr -d "\n")

AC_CHECK_PROG(CARGO,[cargo],[cargo],[no])
test "$CARGO" == "no" && AC_MSG_ERROR([Required program 'cargo' not found.])
AC_CHECK_PROG(RUSTC,[rustc],[rustc],[no])
test "$RUSTC" == "no" && AC_MSG_ERROR([Required program 'rustc' not found.])
AC_CHECK_PROG(ROFI,[rofi],[rofi],[no])
test "$ROFI" == "no" && AC_MSG_ERROR([Required program 'rofi' not found.])


dnl Release build we do.
CARGO_TARGET_DIR=release
AC_SUBST(CARGO_TARGET_DIR)
AC_SUBST(VERSION)
AC_CONFIG_FILES([Makefile])

AC_OUTPUT
