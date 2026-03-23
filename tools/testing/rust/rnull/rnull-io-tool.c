// SPDX-License-Identifier: GPL-2.0

#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <linux/fs.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <time.h>
#include <unistd.h>

static void fill_pattern(uint8_t *buf, size_t len, uint8_t seed)
{
	size_t i;

	for (i = 0; i < len; i++)
		buf[i] = (uint8_t)(seed + (uint8_t)i);
}

static int alloc_aligned(void **buf, size_t len)
{
	int ret = posix_memalign(buf, 4096, len);

	if (ret)
		errno = ret;
	return ret ? -1 : 0;
}

static int open_device(const char *path, int writeable, int direct, int hipri)
{
	int flags = writeable ? O_RDWR : O_RDONLY;

	if (direct || hipri)
		flags |= O_DIRECT;

	return open(path, flags, 0);
}

static ssize_t do_rw(int fd, int write_op, void *buf, size_t len, off_t off, int hipri)
{
	struct iovec iov = {
		.iov_base = buf,
		.iov_len = len,
	};

	if (hipri) {
		if (write_op)
			return pwritev2(fd, &iov, 1, off, RWF_HIPRI);
		return preadv2(fd, &iov, 1, off, RWF_HIPRI);
	}

	if (write_op)
		return pwrite(fd, buf, len, off);
	return pread(fd, buf, len, off);
}

static uint64_t monotonic_ns(void)
{
	struct timespec ts;

	if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
		perror("clock_gettime");
		exit(2);
	}

	return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

static int flush_and_close(int fd, int need_fsync, const char *op)
{
	if (need_fsync && fsync(fd) != 0) {
		perror(op);
		close(fd);
		return -1;
	}

	if (close(fd) != 0) {
		perror("close");
		return -1;
	}

	return 0;
}

static void usage(const char *prog)
{
	fprintf(stderr,
		"Usage:\n"
		"  %s write-pattern <dev> <offset> <bytes> <seed> [hipri]\n"
		"  %s read-verify <dev> <offset> <bytes> <seed> [hipri]\n"
		"  %s read-zero <dev> <offset> <bytes> [hipri]\n"
		"  %s flush <dev>\n"
		"  %s discard <dev> <offset> <bytes>\n"
		"  %s timed-write <dev> <offset> <bytes> <seed> [hipri]\n",
		prog, prog, prog, prog, prog, prog);
}

int main(int argc, char **argv)
{
	const char *cmd;
	const char *path;
	uint64_t offset;
	size_t bytes;
	uint8_t seed = 0;
	int hipri = 0;
	void *buf = NULL;
	int fd = -1;
	ssize_t ret;

	if (argc < 3) {
		usage(argv[0]);
		return 1;
	}

	cmd = argv[1];
	path = argv[2];

	if (!strcmp(cmd, "flush")) {
		fd = open_device(path, 1, 0, 0);
		if (fd < 0) {
			perror("open");
			return 2;
		}
		if (fsync(fd) != 0) {
			perror("fsync");
			close(fd);
			return 2;
		}
		close(fd);
		puts("ok");
		return 0;
	}

	if (!strcmp(cmd, "discard")) {
		uint64_t range[2];

		if (argc < 5) {
			usage(argv[0]);
			return 1;
		}

		offset = strtoull(argv[3], NULL, 0);
		bytes = (size_t)strtoull(argv[4], NULL, 0);
		range[0] = offset;
		range[1] = bytes;

		fd = open_device(path, 1, 0, 0);
		if (fd < 0) {
			perror("open");
			return 2;
		}
		if (ioctl(fd, BLKDISCARD, &range) != 0) {
			perror("BLKDISCARD");
			close(fd);
			return 2;
		}
		close(fd);
		puts("ok");
		return 0;
	}

	if (argc < 5) {
		usage(argv[0]);
		return 1;
	}

	offset = strtoull(argv[3], NULL, 0);
	bytes = (size_t)strtoull(argv[4], NULL, 0);

	if (argc >= 6)
		seed = (uint8_t)strtoul(argv[5], NULL, 0);
	if (argc >= 7)
		hipri = atoi(argv[6]) != 0;

	if (bytes == 0 || (hipri && (bytes % 4096 != 0 || offset % 4096 != 0))) {
		fprintf(stderr, "invalid size or offset for requested mode\n");
		return 1;
	}

	if (alloc_aligned(&buf, bytes) != 0) {
		perror("posix_memalign");
		return 2;
	}

	if (!strcmp(cmd, "write-pattern") || !strcmp(cmd, "timed-write"))
		fill_pattern(buf, bytes, seed);
	else
		memset(buf, 0, bytes);

	fd = open_device(
		path,
		strcmp(cmd, "read-verify") && strcmp(cmd, "read-zero"),
		!strcmp(cmd, "timed-write"),
		hipri
	);
	if (fd < 0) {
		perror("open");
		free(buf);
		return 2;
	}

	if (!strcmp(cmd, "write-pattern")) {
		ret = do_rw(fd, 1, buf, bytes, (off_t)offset, hipri);
		if (ret != (ssize_t)bytes) {
			perror("write-pattern");
			close(fd);
			free(buf);
			return 2;
		}
		if (flush_and_close(fd, 1, "write-pattern") != 0) {
			free(buf);
			return 2;
		}
		fd = -1;
		puts("ok");
	} else if (!strcmp(cmd, "timed-write")) {
		uint64_t start = monotonic_ns();
		uint64_t end;

		ret = do_rw(fd, 1, buf, bytes, (off_t)offset, hipri);
		if (ret != (ssize_t)bytes) {
			perror("timed-write");
			close(fd);
			free(buf);
			return 2;
		}
		if (flush_and_close(fd, 0, "timed-write") != 0) {
			free(buf);
			return 2;
		}
		fd = -1;
		end = monotonic_ns();
		printf("elapsed_ns=%" PRIu64 "\n", end - start);
	} else if (!strcmp(cmd, "read-verify")) {
		uint8_t *expected = malloc(bytes);
		size_t i;

		if (!expected) {
			perror("malloc");
			close(fd);
			free(buf);
			return 2;
		}

		fill_pattern(expected, bytes, seed);
		ret = do_rw(fd, 0, buf, bytes, (off_t)offset, hipri);
		if (ret != (ssize_t)bytes) {
			perror("read-verify");
			free(expected);
			close(fd);
			free(buf);
			return 2;
		}

		for (i = 0; i < bytes; i++) {
			if (((uint8_t *)buf)[i] != expected[i]) {
				fprintf(stderr,
					"verify mismatch at byte %zu: got %u expected %u\n",
					i,
					(unsigned int)((uint8_t *)buf)[i],
					(unsigned int)expected[i]);
				free(expected);
				close(fd);
				free(buf);
				return 3;
			}
		}
		free(expected);
		if (flush_and_close(fd, 0, "read-verify") != 0) {
			free(buf);
			return 2;
		}
		fd = -1;
		puts("ok");
	} else if (!strcmp(cmd, "read-zero")) {
		size_t i;

		ret = do_rw(fd, 0, buf, bytes, (off_t)offset, hipri);
		if (ret != (ssize_t)bytes) {
			perror("read-zero");
			close(fd);
			free(buf);
			return 2;
		}

		for (i = 0; i < bytes; i++) {
			if (((uint8_t *)buf)[i] != 0) {
				fprintf(stderr, "non-zero byte at %zu: %u\n",
					i, (unsigned int)((uint8_t *)buf)[i]);
				close(fd);
				free(buf);
				return 3;
			}
		}
		if (flush_and_close(fd, 0, "read-zero") != 0) {
			free(buf);
			return 2;
		}
		fd = -1;
		puts("ok");
	} else {
		usage(argv[0]);
		close(fd);
		free(buf);
		return 1;
	}

	if (fd >= 0)
		close(fd);
	free(buf);
	return 0;
}
