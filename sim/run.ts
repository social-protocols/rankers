import { objectToSnake } from 'ts-case-convert'

type Item = {
  itemId: number
  parentId?: number
  authorId: string
  createdAt: number
}

type VoteEvent = {
  voteEventId: number
  itemId: number
  userId: string
  vote: number
  rank?: number
  page?: string
  createdAt: number
}

async function getRanking(): Promise<boolean> {
  const url = 'http://localhost:3000/rankings/hn'
  try {
    const response = await fetch(url)
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
  } catch (error) {
    console.error('Fetching error:', error)
    return false
  }
  return true
}

async function postItem(item: Item): Promise<boolean> {
  const url = 'http://localhost:3000/items'
  try {
    const response = await fetch(url, {
      method: 'POST',
      body: JSON.stringify(objectToSnake(item)),
      headers: { 'Content-Type': 'application/json' },
    })

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
  } catch (error) {
    console.error('Fetching error:', error)
    return false
  }
  return true
}

async function postVoteEvent(voteEvent: VoteEvent): Promise<boolean> {
  const url = 'http://localhost:3000/vote_events'
  try {
    const response = await fetch(url, {
      method: 'POST',
      body: JSON.stringify(objectToSnake(voteEvent)),
      headers: { 'Content-Type': 'application/json' },
    })
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
  } catch (error) {
    console.error('Fetching error:', error)
    return false
  }
  return true
}

enum Page {
  Newest = 'Newest',
  HackerNews = 'HackerNews',
  QualityNews = 'QualityNews',
}

enum Action {
  PostItem,
  PostVoteEvent,
  GetRanking,
}

function chooseAction() {
  // TODO: randomize the choice (hard code probabilities)
  return Action.PostItem
}

function chooseRank() {
  // TODO: randomize the choice (Zipf's law)
  return 1
}

function chooseUser() {
  // TODO: randomize the choice (uniform at random)
  return 'user_1'
}

function choosePage() {
  // TODO: randomize the choice (hard code probabilities)
  return Page.HackerNews
}

async function run() {
  console.log('running simulation')

  let itemIdList: number[] = []
  for (let i = 0; i < 10; i++) {
    const itemId = itemIdList.length == 0 ? 1 : Math.max(...itemIdList) + 1
    const item = {
      itemId: itemId,
      authorId: chooseUser(),
      createdAt: Date.now(),
    }
    const success = await postItem(item)
    if (success) {
      itemIdList.push(itemId)
    } else {
      console.error('Failed to post item', itemId)
    }
  }

  console.log('finishing simulation')
}

run()
