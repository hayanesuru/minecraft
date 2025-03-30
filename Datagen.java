package com.jvavav;

import it.unimi.dsi.fastutil.doubles.Double2IntOpenHashMap;
import it.unimi.dsi.fastutil.doubles.DoubleArrayList;
import it.unimi.dsi.fastutil.floats.Float2IntOpenHashMap;
import it.unimi.dsi.fastutil.floats.FloatArrayList;
import it.unimi.dsi.fastutil.ints.IntArrayList;
import it.unimi.dsi.fastutil.ints.IntArrays;
import it.unimi.dsi.fastutil.objects.Object2IntOpenCustomHashMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import net.minecraft.SharedConstants;
import net.minecraft.Util;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.protocol.PacketType;
import net.minecraft.network.protocol.configuration.ConfigurationProtocols;
import net.minecraft.network.protocol.game.GameProtocols;
import net.minecraft.network.protocol.handshake.HandshakeProtocols;
import net.minecraft.network.protocol.login.LoginProtocols;
import net.minecraft.network.protocol.status.StatusProtocols;
import net.minecraft.world.item.BlockItem;
import net.minecraft.world.item.Item;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SupportType;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.material.FlowingFluid;
import net.minecraft.world.level.material.Fluid;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.phys.AABB;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.function.Function;

public class Datagen {
    public static void start() throws IOException {
        var b = new StringBuilder(0x10000);

        b.setLength(0);
        version(b);
        Files.writeString(Path.of("version.txt"), b.toString());

        b.setLength(0);
        registries(b);
        Files.writeString(Path.of("registries.txt"), b.toString());

        b.setLength(0);
        packet(b);
        Files.writeString(Path.of("packet.txt"), b.toString());

        b.setLength(0);
        fluid_state(b);
        Files.writeString(Path.of("fluid_state.txt"), b.toString());

        b.setLength(0);
        block_state(b);
        Files.writeString(Path.of("block_state.txt"), b.toString());

        b.setLength(0);
        entity(b);
        Files.writeString(Path.of("entity.txt"), b.toString());

        b.setLength(0);
        item(b);
        Files.writeString(Path.of("item.txt"), b.toString());

        b.setLength(0);
    }

    private static void version(StringBuilder b) {
        b.append(SharedConstants.getCurrentVersion().getName());
        b.append('\n');
        b.append(Integer.toHexString(SharedConstants.getCurrentVersion().getProtocolVersion()));
        b.append('\n');
    }

    private static void registries(StringBuilder b) {
        for (var registry : BuiltInRegistries.REGISTRY) {
            writeHead(b, registry.key().location().getPath(), registry.size());
            write_registry(b, registry);
        }
    }

    private static <T> void write_registry(StringBuilder b, Registry<T> registry) {
        for (final T t : registry) {
            b.append(Objects.requireNonNull(registry.getKey(t)).getPath());
            b.append('\n');
        }
    }

    private static void entity(StringBuilder b) {
        writeHead(b, "entity_type_height", BuiltInRegistries.ENTITY_TYPE.size());
        writeRl(b, BuiltInRegistries.ENTITY_TYPE, e -> Float.floatToIntBits(e.getHeight()));
        writeHead(b, "entity_type_width", BuiltInRegistries.ENTITY_TYPE.size());
        writeRl(b, BuiltInRegistries.ENTITY_TYPE, e -> Float.floatToIntBits(e.getWidth()));
        writeHead(b, "entity_type_fixed", BuiltInRegistries.ENTITY_TYPE.size());
        writeRl(b, BuiltInRegistries.ENTITY_TYPE, e -> e.getDimensions().fixed() ? 1 : 0);
    }

    private static void fluid_state(StringBuilder b) {
        writeHead(b, "fluid_state", Fluid.FLUID_STATE_REGISTRY.size());
        for (FluidState t : Fluid.FLUID_STATE_REGISTRY) {
            b.append(BuiltInRegistries.FLUID.getKey(t.getType()).getPath());
            if (!t.isEmpty()) {
                if (t.isSource()) {
                    b.append("_s");
                }
                if (t.getValue(FlowingFluid.FALLING)) {
                    b.append("_f");
                }
                b.append('_');
                b.append(ih(t.getAmount()));
            }
            b.append('\n');
        }
        writeHead(b, "fluid_to_block", Fluid.FLUID_STATE_REGISTRY.size());
        for (var f : Fluid.FLUID_STATE_REGISTRY) {
            b.append(ih(Block.BLOCK_STATE_REGISTRY.getId(f.createLegacyBlock())));
            b.append('\n');
        }
        writeHead(b, "fluid_state_level", Fluid.FLUID_STATE_REGISTRY.size());
        for (var f : Fluid.FLUID_STATE_REGISTRY) {
            b.append(ih(f.getAmount()));
            b.append('\n');
        }
        writeHead(b, "fluid_state_falling", Fluid.FLUID_STATE_REGISTRY.size());
        for (var f : Fluid.FLUID_STATE_REGISTRY) {
            b.append(f.isEmpty() ? '0' : f.getValue(FlowingFluid.FALLING) ? '1' : '0');
            b.append('\n');
        }
        writeHead(b, "fluid_state_to_fluid", Fluid.FLUID_STATE_REGISTRY.size());
        for (var f : Fluid.FLUID_STATE_REGISTRY) {
            b.append(ih(BuiltInRegistries.FLUID.getId(f.getType())));
            b.append('\n');
        }
        writeHead(b, "block_to_fluid_state", Block.BLOCK_STATE_REGISTRY.size());
        writeRl(b, Block.BLOCK_STATE_REGISTRY, state -> Fluid.FLUID_STATE_REGISTRY.getId(state.getFluidState()));
    }

    private static void block_state(StringBuilder b) {
        var keys = new Object2IntOpenHashMap<String>();
        var vals = new Object2IntOpenHashMap<String>();
        var kvs = new Object2IntOpenHashMap<IntArrayList>();
        var ps = new Object2IntOpenHashMap<IntArrayList>();
        ps.putIfAbsent(new IntArrayList(), ps.size());
        for (var block : BuiltInRegistries.BLOCK) {
            var p = block.getStateDefinition().getProperties();
            if (p.isEmpty()) {
                continue;
            }

            var list2 = new IntArrayList(p.size());
            for (var x : p) {
                keys.putIfAbsent(x.getName(), keys.size());
                var list = new IntArrayList(x.getPossibleValues().size() + 1);
                list.add(keys.getInt(x.getName()));
                for (var y : x.getPossibleValues()) {
                    var val = Util.getPropertyName(x, y);
                    vals.putIfAbsent(val, vals.size());
                    list.add(vals.getInt(val));
                }
                kvs.putIfAbsent(list, kvs.size());
                list2.add(kvs.getInt(list));
            }
            ps.putIfAbsent(list2, ps.size());
        }

        var keyz = new ObjectArrayList<String>(keys.size());
        keyz.size(keys.size());
        for (var key : keys.object2IntEntrySet()) {
            keyz.set(key.getIntValue(), key.getKey());
        }
        writeHead(b, "block_state_property_key", keyz.size());
        for (var name : keyz) {
            b.append(name);
            b.append('\n');
        }

        var valz = new ObjectArrayList<String>(vals.size());
        valz.size(vals.size());
        for (var val : vals.object2IntEntrySet()) {
            valz.set(val.getIntValue(), val.getKey());
        }
        writeHead(b, "block_state_property_value", valz.size());
        for (var name : valz) {
            b.append(name);
            b.append('\n');
        }

        var kvz = new ObjectArrayList<IntArrayList>(kvs.size());
        kvz.size(kvs.size());
        for (var key : kvs.object2IntEntrySet()) {
            kvz.set(key.getIntValue(), key.getKey());
        }

        var pz = new ObjectArrayList<IntArrayList>(ps.size());
        pz.size(ps.size());
        for (var key : ps.object2IntEntrySet()) {
            pz.set(key.getIntValue(), key.getKey());
        }

        writeHead(b, "block_state_property", kvz.size());
        for (var x : kvz) {
            boolean first = true;
            for (var y : x) {
                if (!first) {
                    b.append(' ');
                }
                first = false;
                b.append(Integer.toHexString(y));
            }
            b.append('\n');
        }
        writeHead(b, "block_state_properties", pz.size());
        for (var x : pz) {
            boolean first = true;
            for (var x1 : x) {
                if (!first) {
                    b.append(' ');
                }
                first = false;
                b.append(ih(x1));
            }
            b.append('\n');
        }

        writeHead(b, "block_state",BuiltInRegistries.BLOCK.size());
        writeRl(b, BuiltInRegistries.BLOCK, block -> {
            var list = new IntArrayList(block.getStateDefinition().getProperties().size());
            for (var prop : block.getStateDefinition().getProperties()) {
                var list2 = new IntArrayList(prop.getPossibleValues().size() + 1);
                list2.add(keys.getInt(prop.getName()));
                for (var x : prop.getPossibleValues()) {
                    list2.add(vals.getInt(Util.getPropertyName(prop, x)));
                }
                list.add(kvs.getInt(list2));
            }
            return ps.getInt(list);
        });


        final int[] prev = {-1};
        writeHead(b, "block_to_default_block_state",BuiltInRegistries.BLOCK.size());
        writeRl(b,BuiltInRegistries.BLOCK, block -> {
            int val = Block.BLOCK_STATE_REGISTRY.getId(block.defaultBlockState());
            int diff = val - prev[0] - 1;
            prev[0] = val;
            return diff;
        });

        prev[0] = -1;
        writeHead(b, "block_item_to_block",BuiltInRegistries.ITEM.size());
        writeRl(b,BuiltInRegistries.ITEM, it -> {
            int val;
            if (it instanceof BlockItem item) {
                val =BuiltInRegistries.BLOCK.getIdOrThrow(item.getBlock());
            } else {
                val =BuiltInRegistries.BLOCK.getIdOrThrow(Blocks.AIR);
            }
            int diff = val - prev[0] - 1;
            prev[0] = val;
            return diff;
        });


        var f32s = new Float2IntOpenHashMap(128);
        f32s.put(0.0f, 0);
        f32s.put(1.0f, 1);
        var f64s = new Double2IntOpenHashMap(128);
        f64s.put(0.0, 0);
        f64s.put(1.0, 1);

        var bs = new Object2IntOpenCustomHashMap<>(IntArrays.HASH_STRATEGY);
        var bx = new IntArrayList(BuiltInRegistries.BLOCK.size());

        for (var block : BuiltInRegistries.BLOCK) {
            f32s.putIfAbsent(block.defaultBlockState().getDestroySpeed(EmptyBlockGetter.INSTANCE, BlockPos.ZERO), f32s.size());
            f32s.putIfAbsent(block.getFriction(), f32s.size());
            f32s.putIfAbsent(block.getSpeedFactor(), f32s.size());
            f32s.putIfAbsent(block.getJumpFactor(), f32s.size());
            f32s.putIfAbsent(block.getExplosionResistance(), f32s.size());

            float a1 = block.defaultBlockState().getDestroySpeed(EmptyBlockGetter.INSTANCE, BlockPos.ZERO);
            float b1 = block.getExplosionResistance();
            float c1 = block.getFriction();
            float d1 = block.getSpeedFactor();
            float e1 = block.getJumpFactor();
            int[] x = new int[]{f32s.get(a1), f32s.get(b1), f32s.get(c1), f32s.get(d1), f32s.get(e1)};
            bs.putIfAbsent(x, bs.size());
            bx.push(bs.getInt(x));
        }
        var bz = new ObjectArrayList<int[]>(bs.size());
        bz.size(bs.size());
        for (var e : bs.object2IntEntrySet()) {
            var k = e.getKey();
            var v = e.getIntValue();
            bz.set(v, k);
        }
        var f32z = new FloatArrayList(f32s.size());
        f32z.size(f32s.size());
        for (var e : f32s.float2IntEntrySet()) {
            var k = e.getFloatKey();
            var v = e.getIntValue();
            f32z.set(v, k);
        }

        writeHead(b, "float32_table", f32z.size());
        for (var e : f32z) {
            b.append(ih(Float.floatToIntBits(e)));
            b.append('\n');
        }

        var shapes = new Object2IntOpenHashMap<List<AABB>>(128);
        for (var block :BuiltInRegistries.BLOCK) {
            if (block.hasDynamicShape()) {
                continue;
            }
            for (var state : block.getStateDefinition().getPossibleStates()) {
                shapes.putIfAbsent(state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).toAabbs(), shapes.size());
                if (state.canOcclude()) {
                    shapes.putIfAbsent(state.getOcclusionShape().toAabbs(), shapes.size());
                }
            }
        }

        var shapes2 = new ObjectArrayList<List<AABB>>(shapes.size());
        shapes2.size(shapes.size());
        for (var e : shapes.object2IntEntrySet()) {
            var k = e.getKey();
            var v = e.getIntValue();
            shapes2.set(v, k);
        }
        for (var shape : shapes2) {
            for (var box : shape) {
                f64s.putIfAbsent(box.minX, f64s.size());
                f64s.putIfAbsent(box.minY, f64s.size());
                f64s.putIfAbsent(box.minZ, f64s.size());
                f64s.putIfAbsent(box.maxX, f64s.size());
                f64s.putIfAbsent(box.maxY, f64s.size());
                f64s.putIfAbsent(box.maxZ, f64s.size());
            }
        }
        var f64z = new DoubleArrayList(f64s.size());
        f64z.size(f64s.size());
        for (var e : f64s.double2IntEntrySet()) {
            var k = e.getDoubleKey();
            var v = e.getIntValue();
            f64z.set(v, k);
        }
        writeHead(b, "float64_table", f64z.size());
        for (var f64 : f64z) {
            b.append(Long.toHexString(Double.doubleToLongBits(f64)));
            b.append('\n');
        }
        writeHead(b, "shape_table", shapes.size());
        for (var e : shapes2) {
            boolean first = true;
            for (var x : e) {
                if (!first) {
                    b.append(' ');
                }
                first = false;

                b.append(ih(f64s.get(x.minX)));
                b.append(' ');
                b.append(ih(f64s.get(x.minY)));
                b.append(' ');
                b.append(ih(f64s.get(x.minZ)));
                b.append(' ');
                b.append(ih(f64s.get(x.maxX)));
                b.append(' ');
                b.append(ih(f64s.get(x.maxY)));
                b.append(' ');
                b.append(ih(f64s.get(x.maxZ)));
            }
            b.append('\n');
        }

        writeHead(b, "block_settings_table#hardness " +
            "blast_resistance slipperiness velocity_multiplier " +
            "jump_velocity_multiplier", bz.size());
        for (var s : bz) {
            boolean first = true;
            for (var x : s) {
                if (!first) {
                    b.append(' ');
                }
                first = false;
                b.append(ih(x));
            }
            b.append('\n');
        }

        writeHead(b, "block_settings",BuiltInRegistries.BLOCK.size());
        writeRl(b, bx, x -> x);

        writeHead(b, "block_state_flags#" +
            "(has_sided_transparency lava_ignitable " +
            "material_replaceable opaque tool_required " +
            "exceeds_cube redstone_power_source " +
            "has_comparator_output)", Block.BLOCK_STATE_REGISTRY.size());
        writeRl(b, Block.BLOCK_STATE_REGISTRY, state ->
            (state.hasAnalogOutputSignal() ? 0b1 : 0) |
                (state.isSignalSource() ? 0b10 : 0) |
                (state.hasLargeCollisionShape() ? 0b100 : 0) |
                (state.requiresCorrectToolForDrops() ? 0b1000 : 0) |
                (state.canOcclude() ? 0b10000 : 0) |
                (state.canBeReplaced() ? 0b100000 : 0) |
                (state.ignitedByLava() ? 0b1000000 : 0) |
                (state.useShapeForLightOcclusion() ? 0b10000000 : 0)
        );

        writeHead(b, "block_state_luminance", Block.BLOCK_STATE_REGISTRY.size());
        writeRl(b, Block.BLOCK_STATE_REGISTRY, BlockBehaviour.BlockStateBase::getLightEmission);

        var bounds = new Object2IntOpenCustomHashMap<>(IntArrays.HASH_STRATEGY);
        var bound2s = new Object2IntOpenCustomHashMap<>(IntArrays.HASH_STRATEGY);
        var bound2x = new IntArrayList(BuiltInRegistries.BLOCK.size());
        bounds.put(new int[0], 0);
        bound2s.put(new int[0], 0);

        for (var block :BuiltInRegistries.BLOCK) {
            var states = block.getStateDefinition().getPossibleStates();
            if (block.hasDynamicShape()) {
                bound2s.putIfAbsent(new int[0], bound2s.size());
                bound2x.push(bound2s.getInt(new int[0]));
                continue;
            }
            int[] z = new int[states.size()];
            for (int i = 0; i < states.size(); i++) {
                final var state = states.get(i);
                int flags1 = 0;
                if (state.isSolidRender()) {
                    flags1 |= 1;
                }
                if (state.isCollisionShapeFullBlock(EmptyBlockGetter.INSTANCE, BlockPos.ZERO)) {
                    flags1 |= 2;
                }
                if (state.propagatesSkylightDown()) {
                    flags1 |= 4;
                }
                if (state.isRedstoneConductor(EmptyBlockGetter.INSTANCE, BlockPos.ZERO)) {
                    flags1 |= 8;
                }
                flags1 |= state.getLightBlock() << 4;

                int flags2 = 0;
                int flags3 = 0;
                int flags4 = 0;
                for (var direction : Direction.values()) {
                    if (state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, direction, SupportType.FULL)) {
                        flags2 |= 1 << direction.get3DDataValue();
                    }
                }
                for (var direction : Direction.values()) {
                    if (state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, direction, SupportType.CENTER)) {
                        flags3 |= 1 << direction.get3DDataValue();
                    }
                }
                for (var direction : Direction.values()) {
                    if (state.isFaceSturdy(EmptyBlockGetter.INSTANCE, BlockPos.ZERO, direction, SupportType.RIGID)) {
                        flags4 |= 1 << direction.get3DDataValue();
                    }
                }
                int flags5 = shapes.getInt(state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).toAabbs());
                int flags6 = shapes.getInt(state.getOcclusionShape().toAabbs());
                int[] x = new int[]{flags1, flags2, flags3, flags4, flags5, flags6};
                bounds.putIfAbsent(x, bounds.size());
                z[i] = bounds.getInt(x);
            }
            int first = z[0];
            if (Arrays.stream(z).allMatch(x -> x == first)) {
                bound2s.putIfAbsent(new int[]{z[0]}, bound2s.size());
                bound2x.push(bound2s.getInt(new int[]{z[0]}));
            } else {
                bound2s.putIfAbsent(z, bound2s.size());
                bound2x.push(bound2s.getInt(z));
            }
        }

        var boundz = new ObjectArrayList<int[]>(bounds.size());
        boundz.size(bounds.size());
        for (var e : bounds.object2IntEntrySet()) {
            var k = e.getKey();
            var v = e.getIntValue();
            boundz.set(v, k);
        }
        var bound2z = new ObjectArrayList<int[]>(bound2s.size());
        bound2z.size(bound2s.size());
        for (var e : bound2s.object2IntEntrySet()) {
            var k = e.getKey();
            var v = e.getIntValue();
            bound2z.set(v, k);
        }
        writeHead(b, "block_state_static_bounds_table#" +
            "(opacity(4) solid_block translucent full_cube " +
            "opaque_full_cube) side_solid_full " +
            "side_solid_center side_solid_rigid " +
            "collision_shape culling_shape", boundz.size());
        for (var bound : boundz) {
            boolean first = true;
            for (var x : bound) {
                if (!first) {
                    b.append(' ');
                }
                first = false;
                b.append(ih(x));
            }
            b.append('\n');
        }

        writeHead(b, "block_state_static_bounds_map", bound2z.size());
        for (var bound : bound2z) {
            boolean first = true;
            for (var x : bound) {
                if (!first) {
                    b.append(' ');
                }
                first = false;
                b.append(ih(x));
            }
            b.append('\n');
        }

        writeHead(b, "block_state_static_bounds",BuiltInRegistries.BLOCK.size());
        writeRl(b, bound2x, val -> val);
    }

    private static void packet(StringBuilder b) {
        for (final var factory : List.of(HandshakeProtocols.SERVERBOUND_TEMPLATE,
            StatusProtocols.CLIENTBOUND_TEMPLATE,
            StatusProtocols.SERVERBOUND_TEMPLATE,
            LoginProtocols.CLIENTBOUND_TEMPLATE,
            LoginProtocols.SERVERBOUND_TEMPLATE,
            ConfigurationProtocols.CLIENTBOUND_TEMPLATE,
            ConfigurationProtocols.SERVERBOUND_TEMPLATE,
            GameProtocols.CLIENTBOUND_TEMPLATE,
            GameProtocols.SERVERBOUND_TEMPLATE
        )
        ) {
            var details = factory.details();

            final int[] sz = new int[]{0};
            details.listPackets((i, j) -> sz[0] += 1);
            writeHead(b, details.flow().id() + "/" + details.id().id(), sz[0]);
            final PacketType<?>[] packets = new PacketType[sz[0]];
            details.listPackets((i, j) -> packets[j] = i);
            for (final PacketType<?> packetType : packets) {
                if (packetType == null) {
                    throw new IllegalStateException("invalid packet type");
                }
                b.append(packetType.id().getPath());
                b.append('\n');
            }
        }
    }

    private static void item(StringBuilder b) {
        writeHead(b, "item_max_count",BuiltInRegistries.ITEM.size());
        writeRl(b,BuiltInRegistries.ITEM, Item::getDefaultMaxStackSize);
    }

    private static void writeHead(StringBuilder b, String name, int size) {
        b.append(';');
        b.append(name);
        b.append(';');
        b.append(ih(size));
        b.append('\n');
    }

    private static <T> void writeRl(StringBuilder b, Iterable<T> registry, Function<T, Integer> function) {
        int ncount = 0;
        int nval = 0;
        for (final T e : registry) {
            var val = function.apply(e);
            if (ncount == 0) {
                ncount = 1;
                nval = val;
            } else if (val == nval) {
                ncount += 1;
            } else if (ncount == 1) {
                b.append(ih(nval));
                b.append('\n');
                nval = val;
            } else {
                b.append('~');
                b.append(ih(ncount));
                b.append(' ');
                b.append(ih(nval));
                b.append('\n');
                ncount = 1;
                nval = val;
            }
        }
        if (ncount == 1) {
            b.append(ih(nval));
            b.append('\n');
        } else if (ncount != 0) {
            b.append('~');
            b.append(ih(ncount));
            b.append(' ');
            b.append(ih(nval));
            b.append('\n');
        }
    }

    private static String ih(int x) {
        return Integer.toHexString(x);
    }
}
