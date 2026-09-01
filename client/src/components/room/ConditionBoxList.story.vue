<script setup lang="ts">
import ConditionBoxList from './ConditionBoxList.vue'
import { conditionBoxListFixture } from './ConditionBoxList.fixture'
</script>

<template>
  <Story title="Room/ConditionBoxList">
    <Variant title="条件あり">
      <ConditionBoxList
        :items="conditionBoxListFixture.items"
        :disabled="false"
        @add="console.log('Add condition')"
        @remove="console.log('Remove condition:', $event)"
        @clear="console.log('Clear conditions')"
      />
    </Variant>

    <Variant title="条件なし">
      <ConditionBoxList :items="[]" :disabled="false" />
    </Variant>

    <Variant title="無効">
      <ConditionBoxList :items="conditionBoxListFixture.items" disabled />
    </Variant>
  </Story>
</template>
