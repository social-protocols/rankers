import { objectToSnake, objectToCamel } from 'ts-case-convert'
import * as PD from 'probability-distributions'

type Item = {
  itemId: number
  parentId?: number
  authorId: string
  createdAt: number
}

type ScoredItem = {
  itemId: number
  rank: number
  page: string
  score: number
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

enum Page {
  Newest = 'Newest',
  HackerNews = 'HackerNews',
  QualityNews = 'QualityNews',
}
  
const API_ENDPOINTS: Record<Page, string> = {
  [Page.Newest]: '/rankings/newest',
  [Page.HackerNews]: '/rankings/hn',
  [Page.QualityNews]: '/rankings/qn',
}

enum Action {
  PostItem,
  PostVoteEvent,
}

function getEndpoint(page: Page): string {
  return API_ENDPOINTS[page]
}

async function getRanking(page: Page): Promise<Array<ScoredItem>> {
  const endpoint = getEndpoint(page)

  const url = 'http://localhost:3000' + endpoint

  const response = await fetch(url)

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  const data = await response.json()
  const ranking = objectToCamel(data)
  
  if (!Array.isArray(ranking)) {
    throw new Error('Ranking is not an array')    
  }

  return ranking as ScoredItem[]
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

function chooseAction() {
  return PD.sample(
    [Action.PostItem, Action.PostVoteEvent],
    1,
    true,
    [0.05, 0.95]
  )[0]
}

function chooseRank(nRanks: number): number {
  const ranks = Array.from({ length: nRanks }, (_, index) => index + 1)

  const normalizationConstant =
    ranks.map(r => 1 / r).reduce((sum, current) => sum + current, 0)

  const probabilities = ranks.map(rank => (1 / normalizationConstant) * (1 / rank))

  return PD.sample(ranks, 1, true, probabilities)[0]
}

function chooseUser(users: Array<number>): string {
  const sample = users[Math.floor(Math.random() * users.length)]
  return `user_${sample}`
}

function choosePage() {
  return PD.sample(
    [Page.QualityNews, Page.HackerNews, Page.Newest],
    1,
    true,
    [0.45, 0.45, 0.1]
  )[0]
}

const sleep = ms => new Promise(res => setTimeout(res, ms))

async function main() {
  console.log('running simulation')

  let itemIdList: number[] = []
  let voteEventIdList: number[] = []
  let users = Array.from({ length: 100 }, (_, index) => index + 1)

  // Seed posts
  for (let i = 0; i < 10; i++) {
    const itemId = itemIdList.length == 0 ? 1 : Math.max(...itemIdList) + 1
    const item = {
      itemId: itemId,
      authorId: chooseUser(users),
      createdAt: Date.now(),
    }
    const success = await postItem(item)
    if (success) {
      itemIdList.push(itemId)
    } else {
      console.error('Failed to post item', itemId)
    }
  }

  // TODO: this is a bit hacky, there should be a better way to handle this
  // Wait for quality news sampling to be initialized
  await sleep(5000)

  for (let i = 0; i < 1000; i++) {
    let action = chooseAction()
    let user = chooseUser(users)
    if (action == Action.PostItem) {
      try {
        const itemId = itemIdList.length == 0 ? 1 : Math.max(...itemIdList) + 1
        const item = {
          itemId: itemId,
          authorId: user,
          createdAt: Date.now(),
        }
        const success = await postItem(item)
        if (success) {
          itemIdList.push(itemId)
          console.info('Submitted item:', itemId)
        }
      } catch {
        console.error('Couldn\'t submit item')
      }
    } else {
      try {
        const voteEventId = voteEventIdList.length == 0 ? 1 : Math.max(...voteEventIdList) + 1
        const page = choosePage()
        const ranking = await getRanking(page)
        const rank = chooseRank(ranking.length)
        const voteEvent = {
          voteEventId: voteEventId,
          itemId: ranking[rank - 1].itemId,
          userId: user,
          vote: 1,
          rank: rank,
          page: page,
          createdAt: Date.now(),
        }
        const success = await postVoteEvent(voteEvent)
        if (success) {
          voteEventIdList.push(voteEventId)
          console.info('Submitted vote event:', voteEventId)
        }
      } catch {
        console.error('Couldn\'t submit vote event')
      }
    }
    await sleep(500)
  }

  console.log('finishing simulation')
}

main()
