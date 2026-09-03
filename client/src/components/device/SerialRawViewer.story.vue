<script setup lang="ts">
import SerialRawViewer from './SerialRawViewer.vue'
import { serialRawViewerFixtures } from './SerialRawViewer.fixture'
</script>

<template>
  <Story title="Device/SerialRawViewer">
    <Variant title="Unsupported">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.unsupported" />
      </div>
    </Variant>

    <Variant title="Idle">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.idle" />
      </div>
    </Variant>

    <Variant title="Selecting port">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.selecting" />
      </div>
    </Variant>

    <Variant title="Running with raw chunks">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.running" />
      </div>
    </Variant>

    <Variant title="Stopping">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.stopping" />
      </div>
    </Variant>

    <Variant title="Disconnected with retained capture">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.disconnected" />
      </div>
    </Variant>

    <Variant title="Error">
      <div class="bg-slate-100 p-6">
        <SerialRawViewer v-bind="serialRawViewerFixtures.error" />
      </div>
    </Variant>
  </Story>
</template>
