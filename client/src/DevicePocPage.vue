<script setup lang="ts">
import SerialRawViewer from './components/device/SerialRawViewer.vue'
import { useDeviceSerialPoc } from './device-poc/useDeviceSerialPoc'

const serialPoc = useDeviceSerialPoc()
</script>

<template>
  <main class="min-h-screen bg-slate-100 px-3 py-6 sm:px-6 sm:py-10">
    <SerialRawViewer
      :state="serialPoc.state.value"
      :chunks="serialPoc.chunks.value"
      :decoded-preview="serialPoc.decodedPreview.value"
      :total-bytes="serialPoc.totalBytes.value"
      :capture-limit-bytes="serialPoc.captureLimitBytes"
      :can-connect="serialPoc.canConnect.value"
      :can-stop="serialPoc.canStop.value"
      :can-clear="serialPoc.canClear.value"
      :can-download="serialPoc.canDownload.value"
      :last-export-sha256="serialPoc.lastExportSha256.value"
      @connect="serialPoc.connect"
      @stop="serialPoc.stop"
      @clear="serialPoc.clearCapture"
      @download-raw="serialPoc.downloadRawCapture"
      @download-metadata="serialPoc.downloadCaptureMetadata"
    />
  </main>
</template>
