import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/cluster'
  },
  {
    path: '/cluster',
    name: 'Cluster',
    component: () => import('@/views/Cluster.vue')
  },
  {
    path: '/topic',
    name: 'Topic',
    component: () => import('@/views/Topic.vue')
  },
  {
    path: '/message',
    name: 'Message',
    component: () => import('@/views/Message.vue')
  },
  {
    path: '/consumer-group',
    name: 'ConsumerGroup',
    component: () => import('@/views/ConsumerGroup.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router