<script setup lang="ts">
import GameTimer from './GameTimer.vue'
import { gameTimerFixtures } from './GameTimer.fixture'
</script>

<template>
  <Story title="Room/GameTimer">
    <Variant title="開始直後">
      <div class="bg-[#eef4ff] p-4">
        <GameTimer v-bind="gameTimerFixtures.justStarted" />
      </div>
    </Variant>

    <Variant title="進行中">
      <div class="bg-[#eef4ff] p-4">
        <GameTimer v-bind="gameTimerFixtures.active" />
      </div>
    </Variant>

    <Variant title="クリア時">
      <div class="bg-[#eef4ff] p-4">
        <GameTimer v-bind="gameTimerFixtures.cleared" />
      </div>
    </Variant>
  </Story>
</template>
