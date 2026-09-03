import { describe, expect, it } from 'vitest'

import { roomPageFixture } from '../RoomPage.fixture'

describe('roomPageFixture', () => {
  it('converts shared mock fixtures into a RoomViewModel', () => {
    expect(roomPageFixture.room).toEqual({
      id: '1411824c-d357-4941-af76-c76cb827dda6',
      number: 1,
      name: '最初の部屋',
    })
    expect(roomPageFixture.serverElapsedMs).toBe(65_000)
    expect(roomPageFixture.selectedProblem?.type).toBe('small')
    expect(roomPageFixture.selectedProblem).toMatchObject({
      id: '22222222-2222-4222-8222-222222222221',
      title: '生年月日',
    })
    expect(roomPageFixture.queryInput.operations).toEqual([
      { control: 'down', count: 16 },
      { control: 'right', count: 2 },
      { control: 'up', count: 1 },
    ])
    expect(roomPageFixture.answerInput.maxLength).toBe(50)
    expect(roomPageFixture.clear.clearedCount).toBe(1)
    expect(roomPageFixture.clear.requiredCount).toBe(4)
  })
})
