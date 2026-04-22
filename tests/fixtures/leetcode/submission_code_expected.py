# Definition for a binary tree node.
# class TreeNode:
#     def __init__(self, val=0, left=None, right=None):
#         self.val = val
#         self.left = left
#         self.right = right
class Solution:
    def __init__(self):
        self.res = 0
        self.rank = 0

    def kthSmallest(self, root: Optional[TreeNode], k: int) -> int:
        self.traverse(root, k)
        return self.res
    
    def traverse(self, root: Optional[TreeNode], k: int):
        if not root:
            return
        
        self.traverse(root.left, k)

        self.rank += 1
        if k == self.rank:
            self.res = root.val
            return

        self.traverse(root.right, k)
