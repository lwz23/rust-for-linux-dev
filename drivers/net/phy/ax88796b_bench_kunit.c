// SPDX-License-Identifier: GPL-2.0

#include <kunit/resource.h>
#include <kunit/test.h>
#include <linux/atomic.h>
#include <linux/ethtool.h>
#include <linux/mii.h>
#include <linux/phy.h>

#define AX88796B_TEST_PHY_ADDR 1
#define AX88796B_TRACE_MAX 128
#define AX88796B_TRACE_BUF_LEN 1024

#define AX88772A_NAME "Asix Electronics AX88772A"
#define AX88772C_NAME "Asix Electronics AX88772C"
#define AX88796B_NAME "Asix Electronics AX88796B"

#define PHY_ID_ASIX_AX88772A 0x003b1861
#define PHY_ID_ASIX_AX88772C 0x003b1881
#define PHY_ID_ASIX_AX88796B 0x003b1841

struct ax88796b_fault {
	int reg;
	int when;
	int err;
};

struct ax88796b_trace_entry {
	char op;
	u8 reg;
	int val;
	bool failed;
};

struct ax88796b_fake_bus {
	u32 phy_id;
	int addr;
	int reset_reads_before_clear;
	int total_reads;
	int total_writes;
	int read_count[PHY_MAX_ADDR];
	int write_count[PHY_MAX_ADDR];
	struct ax88796b_fault read_fault;
	struct ax88796b_fault write_fault;
	unsigned int trace_count;
	struct ax88796b_trace_entry trace[AX88796B_TRACE_MAX];
	u16 regs[PHY_MAX_ADDR];
};

struct ax88796b_test_ctx {
	struct mii_bus *bus;
	struct phy_device *phydev;
	struct ax88796b_fake_bus *fake;
};

static atomic_t ax88796b_bus_id = ATOMIC_INIT(0);

static void ax88796b_trace_append(struct ax88796b_fake_bus *fake, char op,
				  int reg, int val, bool failed)
{
	struct ax88796b_trace_entry *entry;

	if (fake->trace_count >= AX88796B_TRACE_MAX)
		return;

	entry = &fake->trace[fake->trace_count++];
	entry->op = op;
	entry->reg = reg;
	entry->val = val;
	entry->failed = failed;
}

static bool ax88796b_fault_now(struct ax88796b_fault *fault, int reg, int count)
{
	return fault->reg == reg && fault->when == count;
}

static int ax88796b_fake_read(struct mii_bus *bus, int addr, int regnum)
{
	struct ax88796b_fake_bus *fake = bus->priv;
	int count;
	u16 val;

	if (addr != fake->addr)
		return 0xffff;

	if (regnum < 0 || regnum >= PHY_MAX_ADDR)
		return -EINVAL;

	count = ++fake->read_count[regnum];
	fake->total_reads++;

	if (ax88796b_fault_now(&fake->read_fault, regnum, count)) {
		ax88796b_trace_append(fake, 'r', regnum, -fake->read_fault.err,
				      true);
		return -fake->read_fault.err;
	}

	val = fake->regs[regnum];
	ax88796b_trace_append(fake, 'r', regnum, val, false);

	if (regnum == MII_BMCR && (val & BMCR_RESET) &&
	    fake->reset_reads_before_clear > 0) {
		fake->reset_reads_before_clear--;
		if (fake->reset_reads_before_clear == 0)
			fake->regs[regnum] &= ~BMCR_RESET;
	}

	return val;
}

static int ax88796b_fake_write(struct mii_bus *bus, int addr, int regnum, u16 val)
{
	struct ax88796b_fake_bus *fake = bus->priv;
	int count;

	if (addr != fake->addr)
		return 0;

	if (regnum < 0 || regnum >= PHY_MAX_ADDR)
		return -EINVAL;

	count = ++fake->write_count[regnum];
	fake->total_writes++;

	if (ax88796b_fault_now(&fake->write_fault, regnum, count)) {
		ax88796b_trace_append(fake, 'w', regnum, -fake->write_fault.err,
				      true);
		return -fake->write_fault.err;
	}

	fake->regs[regnum] = val;
	ax88796b_trace_append(fake, 'w', regnum, val, false);

	return 0;
}

static void ax88796b_cleanup_ctx(void *data)
{
	struct ax88796b_test_ctx *ctx = data;

	if (!ctx)
		return;

	if (ctx->phydev) {
		phy_device_remove(ctx->phydev);
		phy_device_free(ctx->phydev);
		ctx->phydev = NULL;
	}

	if (ctx->bus) {
		if (ctx->bus->state == MDIOBUS_REGISTERED)
			mdiobus_unregister(ctx->bus);
		mdiobus_free(ctx->bus);
		ctx->bus = NULL;
	}
}

static void ax88796b_seed_identity(struct ax88796b_fake_bus *fake, u32 phy_id)
{
	fake->phy_id = phy_id;
	fake->regs[MII_PHYSID1] = phy_id >> 16;
	fake->regs[MII_PHYSID2] = phy_id & 0xffff;
}

static void ax88796b_seed_basic_linkmodes(struct phy_device *phydev)
{
	linkmode_zero(phydev->supported);
	linkmode_zero(phydev->advertising);

	linkmode_set_bit(ETHTOOL_LINK_MODE_10baseT_Half_BIT, phydev->supported);
	linkmode_set_bit(ETHTOOL_LINK_MODE_10baseT_Full_BIT, phydev->supported);
	linkmode_set_bit(ETHTOOL_LINK_MODE_100baseT_Half_BIT, phydev->supported);
	linkmode_set_bit(ETHTOOL_LINK_MODE_100baseT_Full_BIT, phydev->supported);
	linkmode_set_bit(ETHTOOL_LINK_MODE_Pause_BIT, phydev->supported);
	linkmode_set_bit(ETHTOOL_LINK_MODE_Asym_Pause_BIT, phydev->supported);

	linkmode_copy(phydev->advertising, phydev->supported);
	phydev->is_gigabit_capable = false;
}

static struct ax88796b_test_ctx *
ax88796b_test_ctx_create(struct kunit *test, u32 phy_id)
{
	struct ax88796b_test_ctx *ctx;
	struct mii_bus *bus;
	int ret;

	ctx = kunit_kzalloc(test, sizeof(*ctx), GFP_KERNEL);
	if (!ctx)
		return ERR_PTR(-ENOMEM);

	ret = kunit_add_action_or_reset(test, ax88796b_cleanup_ctx, ctx);
	if (ret)
		return ERR_PTR(ret);

	bus = mdiobus_alloc_size(sizeof(*ctx->fake));
	if (!bus)
		return ERR_PTR(-ENOMEM);

	ctx->bus = bus;
	ctx->fake = bus->priv;
	ctx->fake->addr = AX88796B_TEST_PHY_ADDR;
	ctx->fake->read_fault.reg = -1;
	ctx->fake->write_fault.reg = -1;
	ctx->fake->reset_reads_before_clear = 1;
	ax88796b_seed_identity(ctx->fake, phy_id);

	bus->name = "ax88796b-bench";
	snprintf(bus->id, MII_BUS_ID_SIZE, "ax88796b-%d",
		 atomic_inc_return(&ax88796b_bus_id));
	bus->read = ax88796b_fake_read;
	bus->write = ax88796b_fake_write;
	bus->phy_mask = ~0U;

	for (ret = 0; ret < PHY_MAX_ADDR; ret++)
		bus->irq[ret] = PHY_POLL;

	ret = mdiobus_register(bus);
	if (ret)
		return ERR_PTR(ret);

	ctx->phydev = mdiobus_scan_c22(bus, AX88796B_TEST_PHY_ADDR);
	if (IS_ERR(ctx->phydev))
		return ERR_CAST(ctx->phydev);

	if (!ctx->phydev->drv) {
		ret = device_attach(&ctx->phydev->mdio.dev);
		if (ret < 0)
			return ERR_PTR(ret);
	}

	if (!ctx->phydev->drv)
		return ERR_PTR(-ENODEV);

	return ctx;
}

static void ax88796b_set_read_fault(struct ax88796b_fake_bus *fake, int reg,
				    int when, int err)
{
	fake->read_fault.reg = reg;
	fake->read_fault.when = when;
	fake->read_fault.err = err;
}

static void ax88796b_set_write_fault(struct ax88796b_fake_bus *fake, int reg,
				     int when, int err)
{
	fake->write_fault.reg = reg;
	fake->write_fault.when = when;
	fake->write_fault.err = err;
}

static bool ax88796b_trace_has_exact_write(struct ax88796b_fake_bus *fake, int reg,
					   u16 val)
{
	unsigned int i;

	for (i = 0; i < fake->trace_count; i++) {
		if (fake->trace[i].failed)
			continue;
		if (fake->trace[i].op == 'w' && fake->trace[i].reg == reg &&
		    fake->trace[i].val == val)
			return true;
	}

	return false;
}

static bool ax88796b_trace_has_bmcr_flag_write(struct ax88796b_fake_bus *fake,
						u16 flag)
{
	unsigned int i;

	for (i = 0; i < fake->trace_count; i++) {
		if (fake->trace[i].failed)
			continue;
		if (fake->trace[i].op == 'w' && fake->trace[i].reg == MII_BMCR &&
		    (fake->trace[i].val & flag))
			return true;
	}

	return false;
}

static void ax88796b_format_trace(struct ax88796b_fake_bus *fake, char *buf,
				  size_t len)
{
	unsigned int i;
	size_t off = 0;

	if (!len)
		return;

	for (i = 0; i < fake->trace_count && off < len - 1; i++) {
		const char *sep = i ? "," : "";
		struct ax88796b_trace_entry *entry = &fake->trace[i];

		if (entry->failed) {
			off += scnprintf(buf + off, len - off, "%s%c:%02x:!%d",
					 sep, entry->op, entry->reg, -entry->val);
		} else {
			off += scnprintf(buf + off, len - off, "%s%c:%02x:%04x",
					 sep, entry->op, entry->reg, entry->val);
		}
	}

	if (off == 0)
		scnprintf(buf, len, "-");
}

static void ax88796b_emit_observation(const char *scenario,
				      struct ax88796b_test_ctx *ctx, int ret)
{
	char trace_buf[AX88796B_TRACE_BUF_LEN];
	struct phy_device *phydev = ctx->phydev;
	struct ax88796b_fake_bus *fake = ctx->fake;

	ax88796b_format_trace(fake, trace_buf, sizeof(trace_buf));

	pr_info("ax88796b-observation|scenario=%s|phy_id=0x%08x|ret=%d|speed=%d|duplex=%d|link=%d|autoneg=%d|autoneg_complete=%d|suspended=%d|pause=%d|asym_pause=%d|bmcr_final=0x%04x|reads=%d|writes=%d|trace=%s\n",
		scenario, phydev->phy_id, ret, phydev->speed, phydev->duplex,
		phydev->link, phydev->autoneg, phydev->autoneg_complete,
		phydev->suspended, phydev->pause, phydev->asym_pause,
		fake->regs[MII_BMCR], fake->total_reads, fake->total_writes,
		trace_buf);
}

static int ax88796b_call_read_status(struct phy_device *phydev)
{
	int ret;

	mutex_lock(&phydev->lock);
	ret = phy_read_status(phydev);
	mutex_unlock(&phydev->lock);

	return ret;
}

static void ax88796b_call_link_change_notify(struct phy_device *phydev)
{
	mutex_lock(&phydev->lock);
	phydev->drv->link_change_notify(phydev);
	mutex_unlock(&phydev->lock);
}

static void ax88796b_bind_ax88772a_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));
	KUNIT_ASSERT_NOT_NULL(test, ctx->phydev->drv);
	KUNIT_EXPECT_STREQ(test, AX88772A_NAME, ctx->phydev->drv->name);
	ax88796b_emit_observation("bind_ax88772a", ctx, 0);
}

static void ax88796b_bind_ax88772c_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772C);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));
	KUNIT_ASSERT_NOT_NULL(test, ctx->phydev->drv);
	KUNIT_EXPECT_STREQ(test, AX88772C_NAME, ctx->phydev->drv->name);
	ax88796b_emit_observation("bind_ax88772c", ctx, 0);
}

static void ax88796b_bind_ax88796b_model_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88796B | 0x5);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));
	KUNIT_ASSERT_NOT_NULL(test, ctx->phydev->drv);
	KUNIT_EXPECT_STREQ(test, AX88796B_NAME, ctx->phydev->drv->name);
	ax88796b_emit_observation("bind_ax88796b_model", ctx, 0);
}

static void ax88796b_run_soft_reset_test(struct kunit *test, u32 phy_id,
					 const char *scenario, bool inject_fail)
{
	struct ax88796b_test_ctx *ctx;
	int ret;

	ctx = ax88796b_test_ctx_create(test, phy_id);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->fake->regs[MII_BMCR] = BMCR_ANENABLE;
	ctx->fake->reset_reads_before_clear = 2;

	if (inject_fail)
		ax88796b_set_write_fault(ctx->fake, MII_BMCR, 1, EIO);

	ret = phy_init_hw(ctx->phydev);
	ax88796b_emit_observation(scenario, ctx, ret);

	if (inject_fail) {
		KUNIT_EXPECT_EQ(test, -EIO, ret);
		KUNIT_EXPECT_EQ(test, 1, ctx->fake->total_writes);
		KUNIT_EXPECT_FALSE(test,
				   ax88796b_trace_has_bmcr_flag_write(ctx->fake,
								       BMCR_RESET));
		return;
	}

	KUNIT_EXPECT_EQ(test, 0, ret);
	KUNIT_EXPECT_TRUE(test,
			  ax88796b_trace_has_exact_write(ctx->fake, MII_BMCR, 0));
	KUNIT_EXPECT_TRUE(test,
			  ax88796b_trace_has_bmcr_flag_write(ctx->fake,
							      BMCR_RESET));
	KUNIT_EXPECT_EQ(test, 0, ctx->fake->regs[MII_BMCR] & BMCR_RESET);
}

static void ax88796b_soft_reset_ax88772a_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88772A,
				     "soft_reset_ax88772a", false);
}

static void ax88796b_soft_reset_ax88772c_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88772C,
				     "soft_reset_ax88772c", false);
}

static void ax88796b_soft_reset_ax88796b_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88796B | 0x3,
				     "soft_reset_ax88796b", false);
}

static void ax88796b_soft_reset_ax88772a_write_fail_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88772A,
				     "soft_reset_ax88772a_write_fail", true);
}

static void ax88796b_soft_reset_ax88772c_write_fail_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88772C,
				     "soft_reset_ax88772c_write_fail", true);
}

static void ax88796b_soft_reset_ax88796b_write_fail_test(struct kunit *test)
{
	ax88796b_run_soft_reset_test(test, PHY_ID_ASIX_AX88796B | 0x3,
				     "soft_reset_ax88796b_write_fail", true);
}

static void ax88796b_read_status_link_down_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int ret;
	int link;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->speed = SPEED_UNKNOWN;
	ctx->phydev->duplex = DUPLEX_UNKNOWN;
	ctx->fake->regs[MII_BMSR] = 0;

	ret = ax88796b_call_read_status(ctx->phydev);
	ax88796b_emit_observation("read_status_link_down_ax88772a", ctx, ret);
	link = ctx->phydev->link;

	KUNIT_EXPECT_EQ(test, 0, ret);
	KUNIT_EXPECT_EQ(test, 0, link);
	KUNIT_EXPECT_EQ(test, SPEED_UNKNOWN, ctx->phydev->speed);
	KUNIT_EXPECT_EQ(test, DUPLEX_UNKNOWN, ctx->phydev->duplex);
}

static void ax88796b_read_status_complete_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int autoneg_complete;
	int link;
	int ret;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ax88796b_seed_basic_linkmodes(ctx->phydev);
	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->speed = SPEED_UNKNOWN;
	ctx->phydev->duplex = DUPLEX_UNKNOWN;
	ctx->fake->regs[MII_BMCR] = BMCR_SPEED100 | BMCR_FULLDPLX;
	ctx->fake->regs[MII_BMSR] = BMSR_LSTATUS | BMSR_ANEGCOMPLETE;
	ctx->fake->regs[MII_LPA] = LPA_LPACK | LPA_100FULL |
				   LPA_PAUSE_CAP | LPA_PAUSE_ASYM;

	ret = ax88796b_call_read_status(ctx->phydev);
	ax88796b_emit_observation("read_status_100full_aneg_complete_ax88772a",
				  ctx, ret);
	link = ctx->phydev->link;
	autoneg_complete = ctx->phydev->autoneg_complete;

	KUNIT_EXPECT_EQ(test, 0, ret);
	KUNIT_EXPECT_EQ(test, 1, link);
	KUNIT_EXPECT_EQ(test, 1, autoneg_complete);
	KUNIT_EXPECT_EQ(test, SPEED_100, ctx->phydev->speed);
	KUNIT_EXPECT_EQ(test, DUPLEX_FULL, ctx->phydev->duplex);
	KUNIT_EXPECT_TRUE(test, ctx->phydev->pause);
	KUNIT_EXPECT_TRUE(test, ctx->phydev->asym_pause);
}

static void ax88796b_read_status_aneg_incomplete_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int autoneg_complete;
	int link;
	int ret;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->speed = SPEED_UNKNOWN;
	ctx->phydev->duplex = DUPLEX_UNKNOWN;
	ctx->fake->regs[MII_BMCR] = 0;
	ctx->fake->regs[MII_BMSR] = BMSR_LSTATUS;

	ret = ax88796b_call_read_status(ctx->phydev);
	ax88796b_emit_observation("read_status_10half_aneg_incomplete_ax88772a",
				  ctx, ret);
	link = ctx->phydev->link;
	autoneg_complete = ctx->phydev->autoneg_complete;

	KUNIT_EXPECT_EQ(test, 0, ret);
	KUNIT_EXPECT_EQ(test, 0, link);
	KUNIT_EXPECT_EQ(test, 0, autoneg_complete);
	KUNIT_EXPECT_EQ(test, SPEED_UNKNOWN, ctx->phydev->speed);
	KUNIT_EXPECT_EQ(test, DUPLEX_UNKNOWN, ctx->phydev->duplex);
	KUNIT_EXPECT_FALSE(test, ctx->phydev->pause);
	KUNIT_EXPECT_FALSE(test, ctx->phydev->asym_pause);
}

static void ax88796b_read_status_bmcr_error_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int link;
	int ret;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ax88796b_seed_basic_linkmodes(ctx->phydev);
	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->speed = SPEED_UNKNOWN;
	ctx->phydev->duplex = DUPLEX_UNKNOWN;
	ctx->fake->regs[MII_BMSR] = BMSR_LSTATUS | BMSR_ANEGCOMPLETE;
	ax88796b_set_read_fault(ctx->fake, MII_BMCR, 2, EIO);

	ret = ax88796b_call_read_status(ctx->phydev);
	ax88796b_emit_observation("read_status_bmcr_error_ax88772a", ctx, ret);
	link = ctx->phydev->link;

	KUNIT_EXPECT_EQ(test, -EIO, ret);
	KUNIT_EXPECT_EQ(test, 1, link);
	KUNIT_EXPECT_EQ(test, SPEED_UNKNOWN, ctx->phydev->speed);
	KUNIT_EXPECT_EQ(test, DUPLEX_UNKNOWN, ctx->phydev->duplex);
}

static void ax88796b_read_status_lpa_error_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int ret;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->speed = SPEED_UNKNOWN;
	ctx->phydev->duplex = DUPLEX_UNKNOWN;
	ctx->fake->regs[MII_BMCR] = BMCR_SPEED100 | BMCR_FULLDPLX;
	ctx->fake->regs[MII_BMSR] = BMSR_LSTATUS | BMSR_ANEGCOMPLETE;
	ax88796b_set_read_fault(ctx->fake, MII_LPA, 1, EIO);

	ret = ax88796b_call_read_status(ctx->phydev);
	ax88796b_emit_observation("read_status_lpa_error_ax88772a", ctx, ret);

	KUNIT_EXPECT_EQ(test, -EIO, ret);
	KUNIT_EXPECT_EQ(test, SPEED_100, ctx->phydev->speed);
	KUNIT_EXPECT_EQ(test, DUPLEX_FULL, ctx->phydev->duplex);
	KUNIT_EXPECT_FALSE(test, ctx->phydev->pause);
	KUNIT_EXPECT_FALSE(test, ctx->phydev->asym_pause);
}

static void ax88796b_link_change_notify_nolink_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ax88796b_seed_basic_linkmodes(ctx->phydev);
	ctx->phydev->autoneg = AUTONEG_ENABLE;
	ctx->phydev->state = PHY_NOLINK;
	ctx->fake->regs[MII_BMSR] = 0;
	ctx->fake->regs[MII_BMCR] = 0;
	ctx->fake->reset_reads_before_clear = 1;

	ax88796b_call_link_change_notify(ctx->phydev);
	ax88796b_emit_observation("link_change_notify_nolink_ax88772a", ctx, 0);

	KUNIT_EXPECT_TRUE(test,
			  ax88796b_trace_has_exact_write(ctx->fake, MII_BMCR, 0));
	KUNIT_EXPECT_TRUE(test,
			  ax88796b_trace_has_bmcr_flag_write(ctx->fake,
							      BMCR_ANRESTART));
	KUNIT_EXPECT_GT(test, ctx->fake->total_writes, 2);
}

static void ax88796b_link_change_notify_running_test(struct kunit *test)
{
	struct ax88796b_test_ctx *ctx;
	int reads_before;
	int writes_before;

	ctx = ax88796b_test_ctx_create(test, PHY_ID_ASIX_AX88772A);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->phydev->state = PHY_RUNNING;
	reads_before = ctx->fake->total_reads;
	writes_before = ctx->fake->total_writes;
	ax88796b_call_link_change_notify(ctx->phydev);
	ax88796b_emit_observation("link_change_notify_running_ax88772a", ctx, 0);

	KUNIT_EXPECT_EQ(test, reads_before, ctx->fake->total_reads);
	KUNIT_EXPECT_EQ(test, writes_before, ctx->fake->total_writes);
}

static void ax88796b_run_suspend_resume_test(struct kunit *test, u32 phy_id,
					     const char *scenario)
{
	struct ax88796b_test_ctx *ctx;
	int ret;

	ctx = ax88796b_test_ctx_create(test, phy_id);
	KUNIT_ASSERT_FALSE(test, IS_ERR(ctx));

	ctx->fake->regs[MII_BMCR] = 0;
	ret = phy_suspend(ctx->phydev);
	KUNIT_ASSERT_EQ(test, 0, ret);
	KUNIT_EXPECT_TRUE(test, ctx->phydev->suspended);
	KUNIT_EXPECT_NE(test, 0, ctx->fake->regs[MII_BMCR] & BMCR_PDOWN);

	ret = phy_resume(ctx->phydev);
	ax88796b_emit_observation(scenario, ctx, ret);

	KUNIT_EXPECT_EQ(test, 0, ret);
	KUNIT_EXPECT_FALSE(test, ctx->phydev->suspended);
	KUNIT_EXPECT_EQ(test, 0, ctx->fake->regs[MII_BMCR] & BMCR_PDOWN);
}

static void ax88796b_suspend_resume_ax88772a_test(struct kunit *test)
{
	ax88796b_run_suspend_resume_test(test, PHY_ID_ASIX_AX88772A,
					 "suspend_resume_ax88772a");
}

static void ax88796b_suspend_resume_ax88772c_test(struct kunit *test)
{
	ax88796b_run_suspend_resume_test(test, PHY_ID_ASIX_AX88772C,
					 "suspend_resume_ax88772c");
}

static struct kunit_case ax88796b_bench_cases[] = {
	KUNIT_CASE(ax88796b_bind_ax88772a_test),
	KUNIT_CASE(ax88796b_bind_ax88772c_test),
	KUNIT_CASE(ax88796b_bind_ax88796b_model_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88772a_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88772c_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88796b_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88772a_write_fail_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88772c_write_fail_test),
	KUNIT_CASE(ax88796b_soft_reset_ax88796b_write_fail_test),
	KUNIT_CASE(ax88796b_read_status_link_down_test),
	KUNIT_CASE(ax88796b_read_status_complete_test),
	KUNIT_CASE(ax88796b_read_status_aneg_incomplete_test),
	KUNIT_CASE(ax88796b_read_status_bmcr_error_test),
	KUNIT_CASE(ax88796b_read_status_lpa_error_test),
	KUNIT_CASE(ax88796b_link_change_notify_nolink_test),
	KUNIT_CASE(ax88796b_link_change_notify_running_test),
	KUNIT_CASE(ax88796b_suspend_resume_ax88772a_test),
	KUNIT_CASE(ax88796b_suspend_resume_ax88772c_test),
	{}
};

static struct kunit_suite ax88796b_bench_suite = {
	.name = "ax88796b_bench",
	.test_cases = ax88796b_bench_cases,
};

kunit_test_suite(ax88796b_bench_suite);
