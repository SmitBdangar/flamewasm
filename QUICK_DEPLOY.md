# Quick Deployment Guide

## 🚀 Fastest Way to Deploy (5 Minutes)

### 1. MongoDB Atlas Setup (2 min)
1. Go to https://mongodb.com/cloud/atlas → Sign up (Free)
2. Create cluster → Wait 3-5 minutes
3. Database Access → Create user (save password!)
4. Network Access → Add IP: `0.0.0.0/0` (allow all)
5. Database → Connect → Copy connection string
6. Replace `<password>` in connection string

### 2. Deploy Backend to Railway (2 min)
```bash
# Option A: Via Website
1. Go to https://railway.app → Sign up with GitHub
2. New Project → Deploy from GitHub
3. Select your repo → Set root directory: "backend"
4. Add environment variable:
   MONGODB_URI=your_mongodb_atlas_connection_string
5. Deploy → Copy the URL (e.g., https://xxx.railway.app)

# Option B: Via CLI
npm install -g @railway/cli
railway login
railway init
railway link
railway add --variable MONGODB_URI="your_mongodb_atlas_connection_string"
railway up
railway run npm run seed
```

### 3. Deploy Frontend to Vercel (1 min)
```bash
# Option A: Via Website
1. Go to https://vercel.com → Sign up with GitHub
2. New Project → Import repo
3. Root directory: "frontend"
4. Environment variable:
   REACT_APP_API_URL=https://your-backend.railway.app/api
5. Deploy

# Option B: Via CLI
npm install -g vercel
cd frontend
echo "REACT_APP_API_URL=https://your-backend.railway.app/api" > .env.production
vercel --prod
```

## ✅ Done! Your app is live!

---

## 🎯 Alternative: All-in-One Platforms

### Render (Backend + Frontend)
- **Backend**: https://render.com → New Web Service
- **Frontend**: https://render.com → New Static Site
- Both free tier available

### Heroku (Classic)
- **Backend**: `heroku create` → `git push heroku main`
- **Frontend**: Use buildpack `mars/create-react-app`
- ⚠️ No free tier anymore

---

## 📝 Required Environment Variables

### Backend (.env or Platform Settings)
```
MONGODB_URI=mongodb+srv://user:pass@cluster.mongodb.net/dbname
PORT=5000 (auto-set by platform)
NODE_ENV=production
```

### Frontend (.env.production or Platform Settings)
```
REACT_APP_API_URL=https://your-backend-url.com/api
```

---

## 🔧 Update CORS in Backend

If you get CORS errors, update `backend/server.js`:

```javascript
const corsOptions = {
  origin: [
    'http://localhost:3000',
    'https://your-frontend-domain.vercel.app',
    'https://your-frontend-domain.netlify.app'
  ],
  credentials: true
};
app.use(cors(corsOptions));
```

Or allow all origins (for testing):
```javascript
app.use(cors());
```

---

## 🐛 Common Issues

### "Cannot connect to MongoDB"
- Check MongoDB Atlas IP whitelist (add 0.0.0.0/0)
- Verify connection string has correct password
- Check database user permissions

### "CORS Error"
- Update backend CORS to include frontend URL
- Check environment variables are set

### "API returns 404"
- Verify backend URL is correct
- Check API routes start with `/api/`
- Ensure backend is running

### "Build fails"
- Check Node.js version (use 18.x)
- Verify all dependencies in package.json
- Check build logs for specific errors

---

## 💰 Cost Breakdown

| Service | Cost |
|---------|------|
| MongoDB Atlas | FREE (512MB) |
| Railway | FREE ($5 credit/month) |
| Vercel | FREE (unlimited) |
| **Total** | **$0/month** |

---

## 📚 Full Documentation

See `DEPLOYMENT.md` for detailed instructions for all platforms.

