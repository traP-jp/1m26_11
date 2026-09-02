<script setup lang="ts">
import SerialConnectControl from './SerialConnectControl.vue'
</script>

<template>
  <Story title="Room/SerialConnectControl">
    <Variant title="非対応">
      <SerialConnectControl
        :state="{
          phase: 'unsupported',
          reason: 'api-unavailable',
          message: 'このブラウザはWeb Serial APIに対応していません。',
        }"
        :busy="false"
        :can-connect="false"
        :can-retry="false"
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="未接続">
      <SerialConnectControl
        :state="{ phase: 'idle', message: 'Serial deviceは未接続です。' }"
        :busy="false"
        can-connect
        :can-retry="false"
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="接続中">
      <SerialConnectControl
        :state="{
          phase: 'requesting',
          attempt: 'connect',
          message: 'Serial deviceを選択してください。',
        }"
        busy
        :can-connect="false"
        :can-retry="false"
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="接続済み">
      <SerialConnectControl
        :state="{ phase: 'connected', message: 'Serial deviceから入力を読取り中です。' }"
        :busy="false"
        :can-connect="false"
        :can-retry="false"
        can-disconnect
      />
    </Variant>

    <Variant title="切断">
      <SerialConnectControl
        :state="{
          phase: 'disconnected',
          reason: 'device-disconnected',
          message: 'Serial deviceが切断されました。再接続または代替入力を選択してください。',
        }"
        :busy="false"
        :can-connect="false"
        can-retry
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="接続拒否">
      <SerialConnectControl
        :state="{
          phase: 'error',
          operation: 'request-port',
          message: 'Serial portの選択が拒否またはキャンセルされました。',
        }"
        :busy="false"
        :can-connect="false"
        can-retry
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="読取り失敗">
      <SerialConnectControl
        :state="{
          phase: 'error',
          operation: 'read',
          message: 'Serialの読取りに失敗しました。',
        }"
        :busy="false"
        :can-connect="false"
        can-retry
        :can-disconnect="false"
      />
    </Variant>

    <Variant title="再接続失敗">
      <SerialConnectControl
        :state="{
          phase: 'error',
          operation: 'reconnect',
          message: 'Serialの再接続に失敗しました。',
        }"
        :busy="false"
        :can-connect="false"
        can-retry
        :can-disconnect="false"
      />
    </Variant>
  </Story>
</template>
