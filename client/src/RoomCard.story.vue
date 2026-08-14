<script setup lang="ts">
import RoomCard from './RoomCard.vue'

const roomFixture = {
  id: 'room-01',
  number: 1,
  title: 'Room 01',
  genre: 'General',
  description: 'A cozy room for general discussions.',
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
