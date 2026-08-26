<script setup lang="ts">
import { Disclosure, DisclosureButton, DisclosurePanel } from '@headlessui/vue'

export interface Room {
  id: string
  number: number
  title: string
  genre: string
  description: string
}

withDefaults(
  defineProps<{
    room: Room
    defaultOpen?: boolean
    starting?: boolean
  }>(),
  {
    defaultOpen: false,
    starting: false,
  },
)

const emit = defineEmits<{
  start: [room: Room]
}>()
</script>

<template>
  <Disclosure v-slot="{ open }" as="article" class="room-card" :default-open="defaultOpen">
    <DisclosureButton class="room-card__header" type="button">
      <span class="room-card__number" aria-hidden="true">{{ room.number }}</span>
      <span class="room-card__heading">
        <span class="room-card__genre">{{ room.genre }}</span>
        <span class="room-card__title">{{ room.title }}</span>
      </span>
      <span class="room-card__toggle" aria-hidden="true" :data-open="open">
        <img
          src="@/assets/icon/keyboard_arrow_down_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
          alt=""
          aria-hidden="true"
        />
      </span>
    </DisclosureButton>

    <DisclosurePanel class="room-card__body">
      <p class="room-card__description">{{ room.description }}</p>
      <button
        class="room-card__start"
        type="button"
        :disabled="starting"
        @click="emit('start', room)"
      >
        {{ starting ? '開始しています…' : '開始する' }}
      </button>
    </DisclosurePanel>
  </Disclosure>
</template>

<style scoped>
.room-card {
  width: 100%;
  overflow: hidden;
  border: 1px solid #d8e1ee;
  border-radius: 1.25rem;
  color: #121a2a;
  background: #fff;
  box-shadow: 0 0.25rem 1.75rem rgb(21 34 56 / 12%);
  backdrop-filter: blur(1rem);
}

.room-card__header {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: flex-start;
  gap: 1rem;
  padding: 1.25rem 1.5rem;
  border: 0;
  color: inherit;
  background: transparent;
  cursor: pointer;
  text-align: left;
}

.room-card__heading {
  display: grid;
  flex: 1;
  gap: 0.2rem;
}

.room-card__genre {
  color: hsl(244, 60%, 64%);
  font-size: 0.9rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.room-card__header:hover {
  background: #f5f8fc;
}

.room-card__header:focus-visible {
  outline: 2px solid #5eead4;
  outline-offset: -2px;
}

.room-card__title {
  font-size: 2rem;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.room-card__toggle {
  color: #65758d;
  font-size: 1.25rem;
  font-variation-settings:
    'FILL' 0,
    'wght' 400,
    'GRAD' 0,
    'opsz' 24;
  line-height: 1;
  transition: transform 160ms ease;
}

.room-card__toggle[data-open='true'] {
  transform: rotate(180deg);
}

.room-card__body {
  display: grid;
  gap: 1.25rem;
  padding: 1.25rem 1.5rem 1.5rem;
  border-top: 1px solid #e5eaf2;
}

.room-card__description {
  margin: 0;
  color: #65758d;
  line-height: 1.7;
}

.room-card__start {
  justify-self: end;
  padding: 0.7rem 1.1rem;
  border: 0;
  border-radius: 0.7rem;
  color: #07111f;
  background: hsl(171 30% 78%);
  font-weight: 700;
  cursor: pointer;
}

.room-card__start:hover:not(:disabled) {
  background: hsl(171 35% 70%);
}

.room-card__start:focus-visible {
  outline: 2px solid hsl(171 45% 42%);
  outline-offset: 3px;
}

.room-card__start:disabled {
  cursor: wait;
  opacity: 0.6;
}

.room-card__number {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.01em;
}
</style>
