<script setup lang="ts">
import SerialStatusNotice from './SerialStatusNotice.vue'
</script>

<template>
  <Story title="Room/SerialStatusNotice">
    <Variant title="非対応">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="unsupported" />
      </div>
    </Variant>

    <Variant title="接続拒否">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="denied" />
      </div>
    </Variant>

    <Variant title="接続中">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="connecting" />
      </div>
    </Variant>

    <Variant title="接続済み">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="connected" />
      </div>
    </Variant>

    <Variant title="切断">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="disconnected" />
      </div>
    </Variant>

    <Variant title="再接続失敗">
      <div class="max-w-5xl p-4">
        <SerialStatusNotice status="retry-failed" />
      </div>
    </Variant>
  </Story>
</template>
