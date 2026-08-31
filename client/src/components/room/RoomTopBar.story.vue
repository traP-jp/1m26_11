<script setup lang="ts">
import RoomTopBar from './RoomTopBar.vue'
import { gameTimerFixtures } from './GameTimer.fixture'
</script>

<template>
  <Story title="Room/RoomTopBar">
    <Variant title="デフォルト">
      <div class="bg-gray-100 min-h-screen p-4">
        <RoomTopBar
          roomNumber="01"
          roomName="最初の部屋"
          :serverElapsedMs="gameTimerFixtures.active.serverElapsedMs"
          :active="gameTimerFixtures.active.active"
          @exit="() => console.log('退出処理が実行されましたわ！')"
        />
      </div>
    </Variant>
  </Story>
</template>
