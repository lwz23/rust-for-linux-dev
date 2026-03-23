// SPDX-License-Identifier: GPL-2.0

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long usize;
typedef long ssize;
typedef long s32;
typedef unsigned long long u64;
typedef long long s64;

#define NULL ((void *)0)

#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))

#define SYS_EXIT 1
#define SYS_READ 3
#define SYS_WRITE 4
#define SYS_OPEN 5
#define SYS_CLOSE 6
#define SYS_MOUNT 21
#define SYS_SYNC 36
#define SYS_NANOSLEEP 162
#define SYS_GETDENTS64 217
#define SYS_OPENAT 322
#define SYS_FINIT_MODULE 379
#define SYS_REBOOT 88
#define SYS_SYSLOG 103

#define AT_FDCWD (-100)

#define O_RDONLY 0
#define O_WRONLY 1

#define SYSLOG_ACTION_READ_ALL 3
#define SYSLOG_ACTION_SIZE_BUFFER 10

#define LINUX_REBOOT_MAGIC1 0xfee1dead
#define LINUX_REBOOT_MAGIC2 672274793
#define LINUX_REBOOT_CMD_POWER_OFF 0x4321fedc

#define MAX_CONFIG_FILE 1024
#define MAX_TEXT_FILE 4096
#define MAX_DMESG 131072
#define MAX_POLICIES 16
#define MAX_POLICY_NAME 32
#define MAX_KEY 128
#define MAX_PATH 192

struct linux_dirent64 {
	u64 d_ino;
	s64 d_off;
	u16 d_reclen;
	u8 d_type;
	char d_name[];
};

struct timespec32 {
	s32 tv_sec;
	s32 tv_nsec;
};

struct config {
	char implementation[16];
	char runtime_env[32];
	char driver_module[128];
	char governor_module[128];
	char provider_module[128];
	char expected_scaling_driver[64];
};

struct policy_list {
	int count;
	char names[MAX_POLICIES][MAX_POLICY_NAME];
};

static char config_file_buf[MAX_CONFIG_FILE];
static char text_file_buf[MAX_TEXT_FILE];
static char aux_buf[MAX_TEXT_FILE];
static char dmesg_buf[MAX_DMESG];
static struct config cfg;

static inline long syscall0(long nr)
{
	register long r0 __asm__("r0");
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0" : "=r"(r0) : "r"(r7) : "memory");
	return r0;
}

static inline long syscall1(long nr, long a0)
{
	register long r0 __asm__("r0") = a0;
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0" : "+r"(r0) : "r"(r7) : "memory");
	return r0;
}

static inline long syscall2(long nr, long a0, long a1)
{
	register long r0 __asm__("r0") = a0;
	register long r1 __asm__("r1") = a1;
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0" : "+r"(r0) : "r"(r1), "r"(r7) : "memory");
	return r0;
}

static inline long syscall3(long nr, long a0, long a1, long a2)
{
	register long r0 __asm__("r0") = a0;
	register long r1 __asm__("r1") = a1;
	register long r2 __asm__("r2") = a2;
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0"
			 : "+r"(r0)
			 : "r"(r1), "r"(r2), "r"(r7)
			 : "memory");
	return r0;
}

static inline long syscall4(long nr, long a0, long a1, long a2, long a3)
{
	register long r0 __asm__("r0") = a0;
	register long r1 __asm__("r1") = a1;
	register long r2 __asm__("r2") = a2;
	register long r3 __asm__("r3") = a3;
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0"
			 : "+r"(r0)
			 : "r"(r1), "r"(r2), "r"(r3), "r"(r7)
			 : "memory");
	return r0;
}

static inline long syscall5(long nr, long a0, long a1, long a2, long a3, long a4)
{
	register long r0 __asm__("r0") = a0;
	register long r1 __asm__("r1") = a1;
	register long r2 __asm__("r2") = a2;
	register long r3 __asm__("r3") = a3;
	register long r4 __asm__("r4") = a4;
	register long r7 __asm__("r7") = nr;

	__asm__ volatile("svc 0"
			 : "+r"(r0)
			 : "r"(r1), "r"(r2), "r"(r3), "r"(r4), "r"(r7)
			 : "memory");
	return r0;
}

static usize cstr_len(const char *s)
{
	usize len = 0;

	while (s[len] != '\0')
		len++;

	return len;
}

static int is_space(char ch)
{
	return ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' || ch == '\f' || ch == '\v';
}

static char ascii_lower(char ch)
{
	if (ch >= 'A' && ch <= 'Z')
		return ch + ('a' - 'A');

	return ch;
}

static int str_eq(const char *a, const char *b)
{
	usize i = 0;

	while (a[i] != '\0' && b[i] != '\0') {
		if (a[i] != b[i])
			return 0;
		i++;
	}

	return a[i] == '\0' && b[i] == '\0';
}

static int str_cmp(const char *a, const char *b)
{
	usize i = 0;

	while (a[i] != '\0' && b[i] != '\0') {
		if (a[i] != b[i])
			return (int)((unsigned char)a[i] - (unsigned char)b[i]);
		i++;
	}

	return (int)((unsigned char)a[i] - (unsigned char)b[i]);
}

static int starts_with(const char *s, const char *prefix)
{
	usize i = 0;

	while (prefix[i] != '\0') {
		if (s[i] != prefix[i])
			return 0;
		i++;
	}

	return 1;
}

static int contains_ci(const char *haystack, const char *needle)
{
	usize i;
	usize j;

	if (needle[0] == '\0')
		return 1;

	for (i = 0; haystack[i] != '\0'; i++) {
		for (j = 0; needle[j] != '\0'; j++) {
			if (haystack[i + j] == '\0')
				return 0;
			if (ascii_lower(haystack[i + j]) != ascii_lower(needle[j]))
				break;
		}
		if (needle[j] == '\0')
			return 1;
	}

	return 0;
}

static int contains_word(const char *haystack, const char *needle)
{
	usize i = 0;
	usize nlen = cstr_len(needle);

	while (haystack[i] != '\0') {
		usize j;
		int left_ok;
		int right_ok;

		while (haystack[i] != '\0' && is_space(haystack[i]))
			i++;
		if (haystack[i] == '\0')
			break;

		left_ok = (i == 0) || is_space(haystack[i - 1]);
		right_ok = is_space(haystack[i + nlen]) || haystack[i + nlen] == '\0';

		for (j = 0; j < nlen; j++) {
			if (haystack[i + j] != needle[j])
				break;
		}
		if (j == nlen && left_ok && right_ok)
			return 1;

		while (haystack[i] != '\0' && !is_space(haystack[i]))
			i++;
	}

	return 0;
}

static void mem_copy(char *dst, const char *src, usize len)
{
	usize i;

	for (i = 0; i < len; i++)
		dst[i] = src[i];
}

static void str_copy(char *dst, usize cap, const char *src)
{
	usize i = 0;

	if (cap == 0)
		return;

	while (i + 1 < cap && src[i] != '\0') {
		dst[i] = src[i];
		i++;
	}

	dst[i] = '\0';
}

static void str_append(char *dst, usize cap, const char *src)
{
	usize len = cstr_len(dst);
	usize i = 0;

	if (len >= cap)
		return;

	while (len + 1 < cap && src[i] != '\0') {
		dst[len++] = src[i++];
	}

	dst[len] = '\0';
}

static void str_append_char(char *dst, usize cap, char ch)
{
	usize len = cstr_len(dst);

	if (len + 1 >= cap)
		return;

	dst[len] = ch;
	dst[len + 1] = '\0';
}

static void str_append_long(char *dst, usize cap, long value)
{
	char tmp[32];
	int pos = 0;
	unsigned long magnitude;

	if (value < 0) {
		str_append_char(dst, cap, '-');
		magnitude = (unsigned long)(-value);
	} else {
		magnitude = (unsigned long)value;
	}

	do {
		tmp[pos++] = '0' + (magnitude % 10);
		magnitude /= 10;
	} while (magnitude != 0 && pos < (int)ARRAY_SIZE(tmp));

	while (pos > 0)
		str_append_char(dst, cap, tmp[--pos]);
}

static void normalize_whitespace(char *buf)
{
	usize r = 0;
	usize w = 0;
	int in_space = 0;

	while (buf[r] != '\0') {
		if (is_space(buf[r])) {
			in_space = 1;
			r++;
			continue;
		}

		if (in_space && w != 0)
			buf[w++] = ' ';

		in_space = 0;
		buf[w++] = buf[r++];
	}

	if (w > 0 && buf[w - 1] == ' ')
		w--;

	buf[w] = '\0';
}

static long write_all(int fd, const char *buf, usize len)
{
	usize off = 0;

	while (off < len) {
		long rc = syscall3(SYS_WRITE, fd, (long)(buf + off), len - off);

		if (rc < 0)
			return rc;
		if (rc == 0)
			return -5;
		off += (usize)rc;
	}

	return 0;
}

static void print_text(const char *s)
{
	write_all(1, s, cstr_len(s));
}

static void log_line(const char *message)
{
	print_text("CPUFREQ_DT_TEST: ");
	print_text(message);
	print_text("\n");
}

static void emit_result(const char *key, const char *value)
{
	print_text("CPUFREQ_DT_RESULT: ");
	print_text(key);
	print_text("=");
	print_text(value);
	print_text("\n");
}

static void emit_result_long(const char *key, long value)
{
	char buf[64];

	buf[0] = '\0';
	str_append_long(buf, sizeof(buf), value);
	emit_result(key, buf);
}

static void emit_errno_result(const char *key, long rc)
{
	if (rc < 0)
		emit_result_long(key, -rc);
	else
		emit_result_long(key, rc);
}

static long read_text_file(const char *path, char *buf, usize cap)
{
	long fd;
	usize total = 0;

	if (cap == 0)
		return -22;

	fd = syscall3(SYS_OPEN, (long)path, O_RDONLY, 0);
	if (fd < 0)
		return fd;

	while (total + 1 < cap) {
		long rc = syscall3(SYS_READ, fd, (long)(buf + total), cap - total - 1);

		if (rc < 0) {
			syscall1(SYS_CLOSE, fd);
			return rc;
		}
		if (rc == 0)
			break;
		total += (usize)rc;
	}

	buf[total] = '\0';
	syscall1(SYS_CLOSE, fd);
	return (long)total;
}

static long write_text_file(const char *path, const char *value)
{
	long fd;
	long rc;

	fd = syscall3(SYS_OPEN, (long)path, O_WRONLY, 0);
	if (fd < 0)
		return fd;

	rc = write_all((int)fd, value, cstr_len(value));
	syscall1(SYS_CLOSE, fd);
	return rc;
}

static void build_policy_path(char *out, usize cap, const char *policy, const char *leaf)
{
	out[0] = '\0';
	str_append(out, cap, "/sys/devices/system/cpu/cpufreq/");
	str_append(out, cap, policy);
	if (leaf != NULL && leaf[0] != '\0') {
		str_append_char(out, cap, '/');
		str_append(out, cap, leaf);
	}
}

static void build_policy_key(char *out, usize cap, const char *prefix, const char *policy)
{
	out[0] = '\0';
	str_append(out, cap, prefix);
	str_append_char(out, cap, '.');
	str_append(out, cap, policy);
}

static int scan_policies(struct policy_list *list)
{
	char dirbuf[2048];
	long fd;
	long rc;
	usize off;
	int i;
	int j;

	list->count = 0;

	fd = syscall3(SYS_OPEN, (long)"/sys/devices/system/cpu/cpufreq", O_RDONLY, 0);
	if (fd < 0)
		return (int)fd;

	for (;;) {
		rc = syscall3(SYS_GETDENTS64, fd, (long)dirbuf, sizeof(dirbuf));
		if (rc < 0) {
			syscall1(SYS_CLOSE, fd);
			return (int)rc;
		}
		if (rc == 0)
			break;

		off = 0;
		while (off < (usize)rc) {
			struct linux_dirent64 *entry = (struct linux_dirent64 *)(dirbuf + off);

			if (entry->d_name[0] != '.' && starts_with(entry->d_name, "policy") &&
			    list->count < MAX_POLICIES)
				str_copy(list->names[list->count++], MAX_POLICY_NAME, entry->d_name);

			off += entry->d_reclen;
		}
	}

	syscall1(SYS_CLOSE, fd);

	for (i = 0; i < list->count; i++) {
		for (j = i + 1; j < list->count; j++) {
			char tmp[MAX_POLICY_NAME];

			if (str_cmp(list->names[i], list->names[j]) > 0) {
				mem_copy(tmp, list->names[i], sizeof(tmp));
				mem_copy(list->names[i], list->names[j], sizeof(list->names[i]));
				mem_copy(list->names[j], tmp, sizeof(list->names[j]));
			}
		}
	}

	return list->count;
}

static void sleep_seconds(int seconds)
{
	struct timespec32 ts;

	ts.tv_sec = seconds;
	ts.tv_nsec = 0;
	syscall2(SYS_NANOSLEEP, (long)&ts, 0);
}

static void observe_file(const char *key, const char *path)
{
	long rc = read_text_file(path, text_file_buf, sizeof(text_file_buf));

	if (rc < 0) {
		emit_result(key, "<missing>");
		return;
	}

	normalize_whitespace(text_file_buf);
	emit_result(key, text_file_buf);
}

static void observe_path_exists(const char *key, const char *path)
{
	long fd = syscall3(SYS_OPEN, (long)path, O_RDONLY, 0);

	if (fd < 0) {
		emit_result_long(key, 0);
		return;
	}

	syscall1(SYS_CLOSE, fd);
	emit_result_long(key, 1);
}

static void observe_platform_binding_state(void)
{
	observe_path_exists("platform.device.cpufreq_dt.present",
			    "/sys/bus/platform/devices/cpufreq-dt");
	observe_path_exists("platform.device.cpufreq_dt.driver_link",
			    "/sys/bus/platform/devices/cpufreq-dt/driver");
	observe_path_exists("platform.driver.cpufreq_dt.present",
			    "/sys/bus/platform/drivers/cpufreq-dt");
	observe_path_exists("platform.driver.cpufreq_dt.bound_device",
			    "/sys/bus/platform/drivers/cpufreq-dt/cpufreq-dt");
}

static void emit_matching_lines(const char *key, const char *text, const char *needle)
{
	char out[2048];
	usize out_len = 0;
	int remaining = 20;
	const char *cursor = text;

	out[0] = '\0';

	while (*cursor != '\0' && remaining > 0) {
		const char *line_end = cursor;
		usize line_len;
		usize i;
		int match = 0;

		while (*line_end != '\0' && *line_end != '\n')
			line_end++;

		line_len = (usize)(line_end - cursor);
		if (line_len >= sizeof(aux_buf))
			line_len = sizeof(aux_buf) - 1;

		mem_copy(aux_buf, cursor, line_len);
		aux_buf[line_len] = '\0';

		if (contains_ci(aux_buf, needle))
			match = 1;

		if (match) {
			normalize_whitespace(aux_buf);
			for (i = 0; aux_buf[i] != '\0' && out_len + 2 < sizeof(out); i++)
				out[out_len++] = aux_buf[i];
			if (out_len + 1 < sizeof(out))
				out[out_len++] = ' ';
			remaining--;
		}

		cursor = *line_end == '\n' ? line_end + 1 : line_end;
	}

	if (out_len > 0 && out[out_len - 1] == ' ')
		out_len--;
	out[out_len] = '\0';

	if (out[0] != '\0')
		emit_result(key, out);
}

static void emit_matching_anomalies(const char *key, const char *text)
{
	static const char * const patterns[] = {
		"bug:",
		"warning:",
		"oops:",
		"kasan:",
		"kfence:",
		"kcsan:",
		"ubsan:",
		"use-after-free",
		"double-free",
		"double free",
		"lockdep:",
		"possible recursive locking detected",
		"suspicious rcu usage",
		"refcount_t:",
		"kmemleak:",
		"unreferenced object",
		"bad unlock balance",
	};
	char out[2048];
	usize out_len = 0;
	int remaining = 20;
	const char *cursor = text;

	out[0] = '\0';

	while (*cursor != '\0' && remaining > 0) {
		const char *line_end = cursor;
		usize line_len;
		usize i;
		int match = 0;
		usize p;

		while (*line_end != '\0' && *line_end != '\n')
			line_end++;

		line_len = (usize)(line_end - cursor);
		if (line_len >= sizeof(aux_buf))
			line_len = sizeof(aux_buf) - 1;

		mem_copy(aux_buf, cursor, line_len);
		aux_buf[line_len] = '\0';

		for (p = 0; p < ARRAY_SIZE(patterns); p++) {
			if (contains_ci(aux_buf, patterns[p])) {
				match = 1;
				break;
			}
		}

		if (match) {
			normalize_whitespace(aux_buf);
			for (i = 0; aux_buf[i] != '\0' && out_len + 2 < sizeof(out); i++)
				out[out_len++] = aux_buf[i];
			if (out_len + 1 < sizeof(out))
				out[out_len++] = ' ';
			remaining--;
		}

		cursor = *line_end == '\n' ? line_end + 1 : line_end;
	}

	if (out_len > 0 && out[out_len - 1] == ' ')
		out_len--;
	out[out_len] = '\0';

	if (out[0] != '\0')
		emit_result(key, out);
}

static void scan_dmesg(void)
{
	static const char * const anomaly_patterns[] = {
		"bug:",
		"warning:",
		"oops:",
		"kasan:",
		"kfence:",
		"kcsan:",
		"ubsan:",
		"use-after-free",
		"double-free",
		"double free",
		"lockdep:",
		"possible recursive locking detected",
		"suspicious rcu usage",
		"refcount_t:",
		"kmemleak:",
		"unreferenced object",
		"bad unlock balance",
	};
	long size = syscall3(SYS_SYSLOG, SYSLOG_ACTION_SIZE_BUFFER, 0, 0);
	long rc;
	int anomaly = 0;
	usize p;

	if (size < 0) {
		emit_errno_result("dmesg.read_errno", size);
		return;
	}

	if (size >= (long)sizeof(dmesg_buf))
		size = sizeof(dmesg_buf) - 1;

	rc = syscall3(SYS_SYSLOG, SYSLOG_ACTION_READ_ALL, (long)dmesg_buf, size);
	if (rc < 0) {
		emit_errno_result("dmesg.read_errno", rc);
		return;
	}

	dmesg_buf[rc] = '\0';

	emit_matching_lines("dmesg.cpufreq.head", dmesg_buf, "cpufreq");
	emit_matching_lines("dmesg.cpufreq_dt.head", dmesg_buf, "cpufreq-dt");

	for (p = 0; p < ARRAY_SIZE(anomaly_patterns); p++) {
		if (contains_ci(dmesg_buf, anomaly_patterns[p])) {
			anomaly = 1;
			break;
		}
	}

	emit_result_long("dmesg.anomaly", anomaly);
	if (anomaly)
		emit_matching_anomalies("dmesg.anomaly.head", dmesg_buf);
}

static int parse_config_line(char *line)
{
	char *eq = line;
	char *value;

	while (*eq != '\0' && *eq != '=')
		eq++;
	if (*eq != '=')
		return 0;

	*eq = '\0';
	value = eq + 1;

	if (str_eq(line, "CPUFREQ_DT_IMPLEMENTATION"))
		str_copy(cfg.implementation, sizeof(cfg.implementation), value);
	else if (str_eq(line, "CPUFREQ_DT_ENV"))
		str_copy(cfg.runtime_env, sizeof(cfg.runtime_env), value);
	else if (str_eq(line, "CPUFREQ_DT_DRIVER_MODULE"))
		str_copy(cfg.driver_module, sizeof(cfg.driver_module), value);
	else if (str_eq(line, "CPUFREQ_DT_GOV_MODULE"))
		str_copy(cfg.governor_module, sizeof(cfg.governor_module), value);
	else if (str_eq(line, "CPUFREQ_DT_PROVIDER_MODULE"))
		str_copy(cfg.provider_module, sizeof(cfg.provider_module), value);
	else if (str_eq(line, "CPUFREQ_DT_EXPECTED_SCALING_DRIVER"))
		str_copy(cfg.expected_scaling_driver, sizeof(cfg.expected_scaling_driver), value);

	return 0;
}

static int load_config(void)
{
	long rc;
	char *cursor;

	rc = read_text_file("/etc/cpufreq-dt-test.env", config_file_buf, sizeof(config_file_buf));
	if (rc < 0)
		return (int)rc;

	cursor = config_file_buf;
	while (*cursor != '\0') {
		char *line = cursor;

		while (*cursor != '\0' && *cursor != '\n')
			cursor++;
		if (*cursor == '\n') {
			*cursor = '\0';
			cursor++;
		}

		if (line[0] == '\0')
			continue;

		parse_config_line(line);
	}

	if (cfg.implementation[0] == '\0' || cfg.runtime_env[0] == '\0' ||
	    cfg.driver_module[0] == '\0' || cfg.governor_module[0] == '\0')
		return -22;

	return 0;
}

static int load_module(const char *name, const char *path)
{
	char key[MAX_KEY];
	long fd;
	long rc;

	key[0] = '\0';
	str_append(key, sizeof(key), "module.");
	str_append(key, sizeof(key), name);
	str_append(key, sizeof(key), ".skipped");
	if (path[0] == '\0') {
		emit_result_long(key, 1);
		return 0;
	}

	fd = syscall3(SYS_OPEN, (long)path, O_RDONLY, 0);
	if (fd < 0) {
		key[0] = '\0';
		str_append(key, sizeof(key), "module.");
		str_append(key, sizeof(key), name);
		str_append(key, sizeof(key), ".loaded");
		emit_result_long(key, 0);
		key[0] = '\0';
		str_append(key, sizeof(key), "module.");
		str_append(key, sizeof(key), name);
		str_append(key, sizeof(key), ".err");
		emit_errno_result(key, fd);
		return -1;
	}

	rc = syscall3(SYS_FINIT_MODULE, fd, (long)"", 0);
	syscall1(SYS_CLOSE, fd);

	key[0] = '\0';
	str_append(key, sizeof(key), "module.");
	str_append(key, sizeof(key), name);
	str_append(key, sizeof(key), ".loaded");

	if (rc < 0) {
		emit_result_long(key, 0);
		key[0] = '\0';
		str_append(key, sizeof(key), "module.");
		str_append(key, sizeof(key), name);
		str_append(key, sizeof(key), ".err");
		emit_errno_result(key, rc);
		return -1;
	}

	emit_result_long(key, 1);
	return 0;
}

static void attempt_userspace_roundtrip(const char *policy)
{
	char key[MAX_KEY];
	char path[MAX_PATH];
	char original_governor[64];
	long rc;
	char *space;

	build_policy_path(path, sizeof(path), policy, "scaling_available_governors");
	rc = read_text_file(path, text_file_buf, sizeof(text_file_buf));
	if (rc < 0) {
		build_policy_key(key, sizeof(key), "policy.userspace_available", policy);
		emit_result_long(key, 0);
		return;
	}

	normalize_whitespace(text_file_buf);
	build_policy_key(key, sizeof(key), "policy.userspace_available", policy);
	if (!contains_word(text_file_buf, "userspace")) {
		emit_result_long(key, 0);
		return;
	}

	emit_result_long(key, 1);

	build_policy_path(path, sizeof(path), policy, "scaling_governor");
	rc = read_text_file(path, original_governor, sizeof(original_governor));
	if (rc >= 0)
		normalize_whitespace(original_governor);
	else
		original_governor[0] = '\0';

	rc = write_text_file(path, "userspace\n");
	build_policy_key(key, sizeof(key), "policy.userspace_governor_switch", policy);
	if (rc < 0) {
		emit_result(key, "failed");
		build_policy_key(key, sizeof(key), "policy.userspace_governor_switch_errno", policy);
		emit_errno_result(key, rc);
		return;
	}

	emit_result(key, "ok");

	build_policy_path(path, sizeof(path), policy, "scaling_available_frequencies");
	rc = read_text_file(path, text_file_buf, sizeof(text_file_buf));
	if (rc < 0) {
		build_policy_key(key, sizeof(key), "policy.userspace_setspeed", policy);
		emit_result(key, "unavailable");
		goto restore_governor;
	}

	normalize_whitespace(text_file_buf);
	space = text_file_buf;
	while (*space != '\0' && !is_space(*space))
		space++;
	if (*space != '\0')
		*space = '\0';

	if (text_file_buf[0] == '\0') {
		build_policy_key(key, sizeof(key), "policy.userspace_setspeed", policy);
		emit_result(key, "unavailable");
		goto restore_governor;
	}

	build_policy_path(path, sizeof(path), policy, "scaling_setspeed");
	aux_buf[0] = '\0';
	str_append(aux_buf, sizeof(aux_buf), text_file_buf);
	str_append(aux_buf, sizeof(aux_buf), "\n");
	rc = write_text_file(path, aux_buf);
	build_policy_key(key, sizeof(key), "policy.userspace_setspeed", policy);
	if (rc < 0) {
		emit_result(key, "failed");
		build_policy_key(key, sizeof(key), "policy.userspace_setspeed_errno", policy);
		emit_errno_result(key, rc);
		goto restore_governor;
	}

	sleep_seconds(1);

	emit_result(key, "ok");
	build_policy_key(key, sizeof(key), "policy.userspace_target", policy);
	emit_result(key, text_file_buf);

	build_policy_path(path, sizeof(path), policy, "scaling_cur_freq");
	rc = read_text_file(path, aux_buf, sizeof(aux_buf));
	build_policy_key(key, sizeof(key), "policy.userspace_readback", policy);
	if (rc >= 0) {
		normalize_whitespace(aux_buf);
		emit_result(key, aux_buf);
		goto restore_governor;
	}

	build_policy_path(path, sizeof(path), policy, "cpuinfo_cur_freq");
	rc = read_text_file(path, aux_buf, sizeof(aux_buf));
	if (rc >= 0) {
		normalize_whitespace(aux_buf);
		emit_result(key, aux_buf);
	} else {
		emit_result(key, "<missing>");
	}

restore_governor:
	if (original_governor[0] != '\0') {
		aux_buf[0] = '\0';
		str_append(aux_buf, sizeof(aux_buf), original_governor);
		str_append(aux_buf, sizeof(aux_buf), "\n");
		build_policy_path(path, sizeof(path), policy, "scaling_governor");
		write_text_file(path, aux_buf);
	}
}

static void observe_policy(const char *policy)
{
	static const char * const leaves[] = {
		"scaling_driver",
		"scaling_governor",
		"scaling_available_governors",
		"scaling_available_frequencies",
		"scaling_cur_freq",
		"cpuinfo_cur_freq",
		"related_cpus",
		"affected_cpus",
		"cpuinfo_min_freq",
		"cpuinfo_max_freq",
		"scaling_min_freq",
		"scaling_max_freq",
	};
	static const char * const prefixes[] = {
		"policy.scaling_driver",
		"policy.scaling_governor",
		"policy.scaling_available_governors",
		"policy.scaling_available_frequencies",
		"policy.scaling_cur_freq",
		"policy.cpuinfo_cur_freq",
		"policy.related_cpus",
		"policy.affected_cpus",
		"policy.cpuinfo_min_freq",
		"policy.cpuinfo_max_freq",
		"policy.scaling_min_freq",
		"policy.scaling_max_freq",
	};
	char key[MAX_KEY];
	char path[MAX_PATH];
	usize i;

	build_policy_key(key, sizeof(key), "policy.present", policy);
	emit_result_long(key, 1);

	for (i = 0; i < ARRAY_SIZE(leaves); i++) {
		build_policy_path(path, sizeof(path), policy, leaves[i]);
		build_policy_key(key, sizeof(key), prefixes[i], policy);
		observe_file(key, path);
	}

	attempt_userspace_roundtrip(policy);
}

static int mount_basic_filesystems(void)
{
	long rc;

	rc = syscall5(SYS_MOUNT, (long)"proc", (long)"/proc", (long)"proc", 0, 0);
	if (rc < 0)
		return (int)rc;

	rc = syscall5(SYS_MOUNT, (long)"sysfs", (long)"/sys", (long)"sysfs", 0, 0);
	if (rc < 0)
		return (int)rc;

	rc = syscall5(SYS_MOUNT, (long)"devtmpfs", (long)"/dev", (long)"devtmpfs", 0, 0);
	if (rc < 0)
		return (int)rc;

	return 0;
}

static int wait_for_policies(struct policy_list *list)
{
	int tries;

	for (tries = 0; tries < 30; tries++) {
		int rc = scan_policies(list);

		if (rc > 0)
			return 0;

		sleep_seconds(1);
	}

	list->count = 0;
	return -1;
}

static int main_logic(void)
{
	struct policy_list policies;
	int rc;
	int i;

	log_line("cpufreq-dt automated test init");

	rc = load_config();
	if (rc < 0) {
		emit_errno_result("config.load_errno", rc);
		return 2;
	}

	emit_result("env.name", cfg.runtime_env);
	emit_result("implementation", cfg.implementation);

	rc = mount_basic_filesystems();
	if (rc < 0) {
		emit_errno_result("mount.err", rc);
		return 3;
	}

	if (load_module("provider", cfg.provider_module) < 0)
		return 10;
	if (load_module("governor", cfg.governor_module) < 0)
		return 11;
	if (load_module("driver", cfg.driver_module) < 0)
		return 12;

	sleep_seconds(2);
	observe_platform_binding_state();
	scan_dmesg();

	if (wait_for_policies(&policies) < 0) {
		emit_result_long("policy.root_visible", 0);
		scan_dmesg();
		return 13;
	}

	emit_result_long("policy.root_visible", 1);

	for (i = 0; i < policies.count; i++)
		observe_policy(policies.names[i]);

	emit_result_long("policy.count", policies.count);
	scan_dmesg();

	if (policies.count == 0)
		return 14;

	return 0;
}

void _start(void)
{
	char rc_buf[32];
	int rc = main_logic();

	rc_buf[0] = '\0';
	str_append_long(rc_buf, sizeof(rc_buf), rc);
	print_text("CPUFREQ_DT_GUEST_RC=");
	print_text(rc_buf);
	print_text("\n");
	syscall0(SYS_SYNC);
	syscall4(SYS_REBOOT,
		 LINUX_REBOOT_MAGIC1,
		 LINUX_REBOOT_MAGIC2,
		 LINUX_REBOOT_CMD_POWER_OFF,
		 0);
	syscall1(SYS_EXIT, rc);

	for (;;)
		;
}
