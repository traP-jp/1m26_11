<script setup lang="ts">
import RoomCard from './RoomCard.vue'

const roomFixture = {
  room_id: '1411824c-d357-4941-af76-c76cb827dda6',
  number: 1,
  name: '最初の部屋',
  genre: 'logic',
  description: '動作確認用の問題セットです',
}
</script>

<template>
  <Story title="RoomCard">
    <Variant title="Closed">
      <div style="width: 24rem; padding: 2rem">
        <RoomCard :room="roomFixture" />
      </div>
    </Variant>

    <Variant title="Open">
      <div style="width: 24rem; padding: 2rem">
        <RoomCard :room="roomFixture" default-open />
      </div>
    </Variant>
  </Story>
</template>
