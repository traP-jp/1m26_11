<template>
  <section class="mx-auto grid w-full max-w-xl gap-5 rounded-2xl bg-white p-8 shadow-md">
    <div class="grid gap-2 text-center">
      <h1 class="text-2xl font-bold text-[#121a2a]">ログインしてゲームをはじめる</h1>
      <p class="text-[#65758d]">ゲームに参加するにはログインしてください。</p>
    </div>
    <div class="flex justify-center">
      <slot name="action" />
    </div>
  </section>
</template>
